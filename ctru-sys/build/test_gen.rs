//! This module contains the code to generate layout tests comparing generated
//! Rust bindings to the actual C types defined in libctru. We use [`cpp_build`]
//! to compile helper functions that return the real `sizeof`/`alignof` those types
//! and compare them to the ones generated by `bindgen`.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

use bindgen::callbacks::{
    DeriveInfo, DeriveTrait, FieldInfo, ImplementsTrait, ParseCallbacks, TypeKind,
};
use bindgen::FieldVisibilityKind;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};

use regex::Regex;
use rust_format::{Formatter, RustFmt};

#[derive(Debug)]
pub struct LayoutTestCallbacks(Rc<LayoutTestGenerator>);

impl LayoutTestCallbacks {
    pub fn new() -> (Self, Rc<LayoutTestGenerator>) {
        let generator = Rc::new(LayoutTestGenerator::new());
        (Self(Rc::clone(&generator)), generator)
    }
}

impl ParseCallbacks for LayoutTestCallbacks {
    fn header_file(&self, filename: &str) {
        self.0.headers.borrow_mut().push(filename.to_string());
    }

    fn add_derives(&self, info: &DeriveInfo<'_>) -> Vec<String> {
        if let TypeKind::Union = info.kind {
            // layout tests don't handle unions for now, just skip it
            println!(
                "cargo:warning=Skipping layout tests for union {}",
                info.name,
            );
            self.0.blocklist_type(info.name);
        }

        Vec::new()
    }

    fn blocklisted_type_implements_trait(
        &self,
        name: &str,
        _derive_trait: DeriveTrait,
    ) -> Option<ImplementsTrait> {
        self.0.blocklist_type(name);
        None
    }

    fn field_visibility(&self, info: FieldInfo<'_>) -> Option<FieldVisibilityKind> {
        self.0
            .struct_fields
            .borrow_mut()
            .entry(info.type_name.to_string())
            .or_default()
            .insert(info.field_name.to_string());

        None
    }
}

#[derive(Debug)]
pub struct LayoutTestGenerator {
    struct_fields: RefCell<BTreeMap<String, BTreeSet<String>>>,
    blocklist: RefCell<Vec<(Regex, Option<Regex>)>>,
    headers: RefCell<Vec<String>>,
}

impl LayoutTestGenerator {
    fn new() -> Self {
        Self {
            struct_fields: RefCell::default(),
            blocklist: RefCell::default(),
            headers: RefCell::default(),
        }
    }

    pub fn blocklist_type(&self, pattern: &str) -> &Self {
        self.blocklist
            .borrow_mut()
            .push((Regex::new(pattern).unwrap(), None));
        self
    }

    pub fn blocklist_field(&self, struct_pattern: &str, field_pattern: &str) -> &Self {
        self.blocklist.borrow_mut().push((
            Regex::new(struct_pattern).unwrap(),
            Some(Regex::new(field_pattern).unwrap()),
        ));
        self
    }
    pub fn generate_layout_tests(
        &self,
        output_path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;

        // Since quote! tokenizes its input, it would result in invalid C++ for
        // the `#include` directives (they would be missing whitespace/newlines),
        // so we basically need to drop in the include headers here "manually" by
        // writing them into the cpp! macro invocation.
        file.write_all("cpp! {{".as_bytes())?;
        for included_file in self.headers.borrow().iter() {
            writeln!(file, "    #include \"{included_file}\"")?;
        }
        file.write_all("}}\n".as_bytes())?;

        let test_tokens = RustFmt::default().format_tokens(self.build_tests())?;
        file.write_all(test_tokens.as_bytes())?;

        Ok(())
    }

    fn build_tests(&self) -> TokenStream {
        let mut output = TokenStream::new();

        for struct_name in self.struct_fields.borrow().keys() {
            if self
                .blocklist
                .borrow()
                .iter()
                .any(|(pat, field)| field.is_none() && pat.is_match(struct_name))
            {
                println!("cargo:warning=Skipping layout tests for {struct_name}",);
                continue;
            }

            output.append_all(self.build_struct_test(struct_name));
        }

        output
    }

    fn build_struct_test(&self, struct_name: &str) -> proc_macro2::TokenStream {
        let name = format_ident!("{struct_name}");
        let test_name = format_ident!("layout_test_{name}");

        let mut field_tests = Vec::new();
        field_tests.push(build_assert_eq(
            quote!(size_of!(#name)),
            quote!(sizeof(#name)),
        ));
        field_tests.push(build_assert_eq(
            quote!(align_of!(#name)),
            quote!(alignof(#name)),
        ));

        let struct_fields = self.struct_fields.borrow();
        if let Some(fields) = struct_fields.get(struct_name) {
            for field in fields {
                if self
                    .blocklist
                    .borrow()
                    .iter()
                    .any(|(struct_pat, field_pat)| match field_pat {
                        Some(field_pat) => {
                            struct_pat.is_match(struct_name) && field_pat.is_match(field)
                        }
                        None => false,
                    })
                {
                    println!("cargo:warning=Skipping layout tests for {struct_name}::{field}",);
                    continue;
                }

                let field = format_ident!("{field}");

                field_tests.push(build_assert_eq(
                    quote!(size_of!(#name::#field)),
                    quote!(sizeof(#name::#field)),
                ));

                field_tests.push(build_assert_eq(
                    quote!(align_of!(#name::#field)),
                    quote!(alignof(#name::#field)),
                ));

                field_tests.push(build_assert_eq(
                    quote!(offset_of!(#name, #field)),
                    quote!(offsetof(#name, #field)),
                ));
            }
        }

        quote! {
            #[test]
            fn #test_name() {
                #(#field_tests);*
            }
        }
    }
}

fn build_assert_eq(rust_lhs: TokenStream, cpp_rhs: TokenStream) -> TokenStream {
    quote! {
        assert_eq!(
            #rust_lhs,
            cpp!(unsafe [] -> usize as "size_t" { return #cpp_rhs; }),
            "{} != {}",
            stringify!(#rust_lhs),
            stringify!(#cpp_rhs),
        );
    }
}
