//! Definitions of `x86_64` functionality.

use structures::idt::InterruptDescriptorTable;

mod boot;
#[cfg(feature = "debugcon")]
mod debugcon;
mod memory;
mod structures;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
