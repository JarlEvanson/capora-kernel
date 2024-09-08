//! Module controlling booting for the kernel on `x86_64`, parsing bootloader structures and
//! transferring to [`kmain`].

use crate::kmain;

/// The entry point for `x86_64` specific setup.
#[export_name = "_start"]
pub unsafe extern "C" fn karchmain() -> ! {
    kmain()
}
