//! Module controlling booting for the kernel on `x86_64`, parsing bootloader structures and
//! transferring to [`kmain`].

use crate::kmain;

#[cfg(feature = "capora-boot-api")]
pub mod capora_boot_stub;

#[cfg(feature = "limine-boot-api")]
pub mod limine;

/// The entry point for bootloader-independent `x86_64` specific setup.
pub fn karchmain() -> ! {
    kmain()
}
