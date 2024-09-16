//! Driver for `x86_64` logging capabilities.

use core::fmt::Write;

#[cfg(not(any(feature = "debugcon")))]
compile_error!("Kernel logging must have an output method");

/// Initializes architecture specific logging mechanisms.
pub fn init_arch_logger(_logger: &mut ArchitectureLogger) {}

/// An architecture specific logger.
pub struct ArchitectureLogger {}

impl ArchitectureLogger {
    /// Creates a new uninitialzed [`ArchitectureLogger`].
    pub const fn new() -> Self {
        Self {}
    }
}

impl log::Log for ArchitectureLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        #[cfg(feature = "debugcon")]
        let _ = writeln!(
            crate::arch::x86_64::debugcon::acquire_debugcon(),
            "[{:?}] {}",
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}
