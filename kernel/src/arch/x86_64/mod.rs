//! Definitions of `x86_64` functionality.

use structures::idt::InterruptDescriptorTable;

mod boot;
#[cfg(feature = "debugcon-logging")]
mod debugcon;
#[cfg(feature = "logging")]
pub mod logging;
mod memory;
mod structures;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
