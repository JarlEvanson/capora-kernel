//! Capability based microkernel.

#![no_std]
#![no_main]

pub mod cells;

/// Handler of all panics.
#[cfg_attr(not(test), panic_handler)]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop()
    }
}
