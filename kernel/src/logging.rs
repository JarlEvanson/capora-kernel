//! Driver for the logging capabilities of kernel.

use crate::{
    arch::logging::{init_arch_logger, ArchitectureLogger},
    spinlock::Spinlock,
};

static LOCK: Spinlock<ArchitectureLogger> = Spinlock::new(ArchitectureLogger::new());

/// Initializes kernel logging.
pub fn init_logging() {
    init_arch_logger(&mut LOCK.lock());

    log::set_logger(&Logger {}).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        LOCK.lock().enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        LOCK.lock().log(record);
    }

    fn flush(&self) {
        LOCK.lock().flush();
    }
}
