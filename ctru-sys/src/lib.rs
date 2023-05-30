#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod result;

mod bindings;

pub use bindings::*;
pub use result::*;

// static inline functions re-written
#[inline]
pub unsafe fn getThreadLocalStorage() -> *mut libc::c_void {
    let ret;
	core::arch::asm!(
        "mrc p15, 0, {ptr}, c13, c0, 3",
        ptr = out(reg) ret);
	ret
}

#[inline]
pub unsafe fn getThreadCommandBuffer() -> *mut u32 {
    (getThreadLocalStorage() as *mut u8).add(0x80) as *mut u32
}

#[inline]
pub unsafe fn getThreadStaticBuffer() -> *mut u32 {
    (getThreadLocalStorage() as *mut u8).add(0x180) as *mut u32
}

#[inline]
pub const fn IPC_MakeHeader(command_id: u16, normal_params: usize, translate_params: usize) -> u32 {
	return ((command_id as u32) << 16) | (((normal_params as u32) & 0x3F) << 6) | (((translate_params as u32) & 0x3F) << 0);
}


/// In lieu of a proper errno function exposed by libc
/// (<https://github.com/rust-lang/libc/issues/1995>).
pub unsafe fn errno() -> s32 {
    *__errno()
}
