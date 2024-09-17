//! Driver for `x86_64` logging capabilities.

#[cfg(any(feature = "debugcon-logging", feature = "serial-logging"))]
use core::fmt::Write;

#[cfg(feature = "serial-logging")]
use crate::{
    arch::x86_64::serial::{
        DmaMode, DmaTriggerLevel, FifoControl, InterruptEnable, LineControl, SerialPort,
    },
    spinlock::Spinlock,
};

#[cfg(not(any(feature = "debugcon-logging", feature = "serial-logging")))]
compile_error!("Kernel logging must have an output method");

/// Initializes architecture specific logging mechanisms.
pub fn init_arch_logger(logger: &mut ArchitectureLogger) {
    #[cfg(feature = "serial-logging")]
    {
        let mut serial_port = logger.serial_port.lock();
        serial_port.set_interrupt_enable(InterruptEnable::new());
        serial_port.set_line_control(LineControl::new().set_dlab(true));
        serial_port.set_divisor(1);
        serial_port.set_line_control(LineControl::new());
        serial_port.set_fifo_control(
            FifoControl::new()
                .enable_fifo(true)
                .reset_receive_fifo(true)
                .reset_transmit_fifo(true)
                .dma_mode(DmaMode::MultiByte)
                .trigger_level(DmaTriggerLevel::Bytes14),
        );
    }
}

/// An architecture specific logger.
pub struct ArchitectureLogger {
    #[cfg(feature = "serial-logging")]
    serial_port: Spinlock<SerialPort>,
}

impl ArchitectureLogger {
    /// Creates a new uninitialzed [`ArchitectureLogger`].
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "serial-logging")]
            serial_port: unsafe {
                crate::spinlock::Spinlock::new(crate::arch::x86_64::serial::SerialPort::new(0x3f8))
            },
        }
    }
}

impl log::Log for ArchitectureLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        #[cfg(feature = "debugcon-logging")]
        let _ = writeln!(
            crate::arch::x86_64::debugcon::acquire_debugcon(),
            "[{:?}] {}",
            record.level(),
            record.args()
        );

        #[cfg(feature = "serial-logging")]
        let _ = writeln!(
            self.serial_port.lock(),
            "[{:?}] {}",
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}
