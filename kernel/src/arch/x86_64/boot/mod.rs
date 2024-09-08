//! Module controlling booting for the kernel on `x86_64`, parsing bootloader structures and
//! transferring to [`kmain`].

use crate::{
    arch::x86_64::{
        structures::idt::{load_idt, InterruptStackFrame},
        IDT,
    },
    kmain,
};

#[cfg(feature = "capora-boot-api")]
pub mod capora_boot_stub;

#[cfg(feature = "limine-boot-api")]
pub mod limine;

/// The entry point for bootloader-independent `x86_64` specific setup.
pub fn karchmain() -> ! {
    setup_idt();

    kmain()
}

pub fn setup_idt() {
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    idt.double_fault.set_handler_fn(double_fault_handler);

    unsafe { load_idt(idt) }
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, code: u64) -> ! {
    loop {}
}
