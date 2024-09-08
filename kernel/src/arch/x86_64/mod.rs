//! Definitions of `x86_64` functionality.

use structures::idt::InterruptDescriptorTable;

mod boot;
mod memory;
mod structures;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
