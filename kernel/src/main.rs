//! Capability based microkernel.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod arch;
pub mod cells;
#[cfg(feature = "logging")]
pub mod logging;
pub mod spinlock;

/// The architecture independent kernel entry point for the primary CPU.
///
/// This is called by the architecture dependent entry code.
pub fn kmain() -> ! {
    loop {}
}

/// Handler of all panics.
#[cfg_attr(not(test), panic_handler)]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop()
    }
}
