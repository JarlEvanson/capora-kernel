//! Architecture dependent code.

#[cfg(all(feature = "debugcon-logging", not(target_arch = "x86_64")))]
compile_error!("Feature `debugcon-logging` is not available on non-`x86_64` architectures");

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
