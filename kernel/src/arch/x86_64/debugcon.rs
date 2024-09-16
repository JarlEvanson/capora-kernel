//! Driver for the debugcon device.

use crate::spinlock::{Spinlock, SpinlockGuard};

static LOCK: Spinlock<Debugcon> = Spinlock::new(Debugcon());

/// Acquires the debugcon driver.
pub fn acquire_debugcon() -> SpinlockGuard<'static, Debugcon> {
    LOCK.lock()
}

pub struct Debugcon();

impl Debugcon {
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") 0xe9,
                in("al") byte,
            )
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            core::arch::asm!(
                "rep outsb",
                in("dx") 0xe9,
                inout("rsi") bytes.as_ptr() => _,
                inout("rcx") bytes.len() => _,
            )
        }
    }
}

impl core::fmt::Write for Debugcon {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_bytes(s.as_bytes());

        Ok(())
    }
}
