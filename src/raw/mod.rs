#![allow(non_camel_case_types)]
#![allow(overflowing_literals)]

pub mod console;
pub mod gfx;
pub mod linear;
pub mod os;
pub mod sdmc;
pub mod srv;
pub mod svc;
pub mod types;

pub mod services;

pub use self::types::*;

extern crate core;
use core::option::Option;

#[repr(u8)]
pub enum c_void {
    __variant1,
    __variant2
}

pub type ThreadFunc = Option<extern "C" fn(arg1: *mut c_void) -> ()>;