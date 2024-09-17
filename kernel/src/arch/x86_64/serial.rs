//! Driver for the serial port device.

use core::fmt;

pub struct SerialPort {
    io_port: u16,
}

impl SerialPort {
    pub const unsafe fn new(io_port: u16) -> Self {
        Self { io_port }
    }

    pub fn set_interrupt_enable(&mut self, interrupt_enable: InterruptEnable) {
        outb(self.interrupt_enable_port(), interrupt_enable.0)
    }

    pub fn get_interrupt_enable(&self) -> InterruptEnable {
        InterruptEnable(inb(self.interrupt_enable_port()))
    }

    pub fn get_interrupt_status(&self) -> InterruptStatus {
        InterruptStatus(inb(self.interrupt_status_port()))
    }

    pub fn set_fifo_control(&mut self, fifo_control: FifoControl) {
        outb(self.fifo_control_port(), fifo_control.0)
    }

    pub fn set_line_control(&mut self, line_control: LineControl) {
        outb(self.line_control_port(), line_control.0)
    }

    pub fn get_line_control(&self) -> LineControl {
        LineControl(inb(self.line_control_port()))
    }

    pub fn set_divisor(&mut self, divisor: u16) {
        outb(self.divisor_low_port(), divisor as u8);
        outb(self.divisor_high_port(), (divisor >> 8) as u8);
    }

    pub fn get_line_status(&self) -> LineStatus {
        LineStatus(inb(self.line_status_port()))
    }

    pub fn get_divisor(&self) -> u16 {
        let low = inb(self.divisor_low_port());
        let high = inb(self.divisor_high_port());

        ((high as u16) << 8) | (low as u16)
    }

    pub fn write_byte(&mut self, byte: u8) {
        while self.try_write_byte(byte).is_err() {}
    }

    pub fn try_write_byte(&mut self, byte: u8) -> Result<(), u8> {
        let line_status = self.get_line_status();
        if line_status.output_empty() {
            outb(self.transmit_port(), byte);
            Ok(())
        } else {
            Err(byte)
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        loop {
            let result = self.try_read_byte();
            match result {
                Ok(byte) => return byte,
                Err(_) => continue,
            }
        }
    }

    pub fn try_read_byte(&mut self) -> Result<u8, LineStatus> {
        let line_status = self.get_line_status();
        if !line_status.error_set() {
            let byte = inb(self.recieve_port());
            Ok(byte)
        } else {
            Err(line_status)
        }
    }

    fn recieve_port(&self) -> u16 {
        self.io_port
    }

    fn transmit_port(&self) -> u16 {
        self.io_port
    }

    fn interrupt_enable_port(&self) -> u16 {
        self.io_port + 1
    }

    fn interrupt_status_port(&self) -> u16 {
        self.io_port + 2
    }

    fn fifo_control_port(&self) -> u16 {
        self.io_port + 2
    }

    fn line_control_port(&self) -> u16 {
        self.io_port + 3
    }

    fn modem_control_port(&self) -> u16 {
        self.io_port + 4
    }

    fn line_status_port(&self) -> u16 {
        self.io_port + 5
    }

    fn modem_status_port(&self) -> u16 {
        self.io_port + 6
    }

    fn scratch_pad_port(&self) -> u16 {
        self.io_port + 7
    }

    fn divisor_low_port(&self) -> u16 {
        self.io_port
    }

    fn divisor_high_port(&self) -> u16 {
        self.io_port + 1
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InterruptEnable(u8);

impl InterruptEnable {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn set_receive(self, enable: bool) -> Self {
        Self((self.0 & !0b1) | (enable as u8))
    }

    pub const fn set_write(self, enable: bool) -> Self {
        Self((self.0 & !0b10) | ((enable as u8) << 1))
    }

    pub const fn set_error(self, enable: bool) -> Self {
        Self((self.0 & !0b100) | ((enable as u8) << 2))
    }

    pub const fn set_modem_status(self, enable: bool) -> Self {
        Self((self.0 & !0b1000) | ((enable as u8) << 3))
    }

    pub const fn receive_enabled(self) -> bool {
        self.0 & 0b1 == 0b1
    }

    pub const fn write_enabled(self) -> bool {
        (self.0 >> 1) & 0b1 == 0b1
    }

    pub const fn error_enabled(self) -> bool {
        (self.0 >> 2) & 0b1 == 0b1
    }

    pub const fn modem_status_enabled(self) -> bool {
        (self.0 >> 3) & 0b1 == 0b1
    }
}

impl fmt::Debug for InterruptEnable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("InterruptEnable");

        debug_struct.field("receive_enabled", &self.receive_enabled());
        debug_struct.field("write_enabled", &self.write_enabled());
        debug_struct.field("error_enabled", &self.error_enabled());
        debug_struct.field("modem_status_enabled", &self.modem_status_enabled());

        debug_struct.finish()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InterruptStatus(u8);

impl InterruptStatus {
    pub const fn pending(self) -> bool {
        self.0 & 0b1 == 0b1
    }

    pub const fn pending_interrupt(self) -> u8 {
        (self.0 >> 1) & 0b111
    }
}

impl fmt::Debug for InterruptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("InterruptStatus");

        debug_struct.field("pending", &self.pending());
        debug_struct.field("pending_interrupt", &self.pending_interrupt());

        debug_struct.finish()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FifoControl(u8);

impl FifoControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn enable_fifo(self, enable: bool) -> Self {
        Self((self.0 & !0b1) | (enable as u8))
    }

    pub const fn reset_receive_fifo(self, reset: bool) -> Self {
        Self((self.0 & !0b10) | ((reset as u8) << 1))
    }

    pub const fn reset_transmit_fifo(self, reset: bool) -> Self {
        Self((self.0 & 0b100) | ((reset as u8) << 2))
    }

    pub const fn dma_mode(self, dma_mode: DmaMode) -> Self {
        Self((self.0 & 0b1000) | ((dma_mode as u8) << 3))
    }

    pub const fn trigger_level(self, dma_trigger_level: DmaTriggerLevel) -> Self {
        Self((self.0 & 0b11000000) | ((dma_trigger_level as u8) << 6))
    }
}

pub enum DmaMode {
    SingleByte = 0,
    MultiByte = 1,
}

pub enum DmaTriggerLevel {
    Byte1 = 0,
    Bytes4 = 1,
    Bytes8 = 2,
    Bytes14 = 3,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct LineControl(u8);

impl LineControl {
    pub const fn new() -> Self {
        Self(0)
            .set_data_bits(DataBits::Bits8)
            .set_stop_bits(StopBits::OneBit)
            .set_parity(Parity::Disabled)
            .set_break(false)
            .set_dlab(false)
    }

    pub const fn set_data_bits(self, data_bits: DataBits) -> Self {
        Self((self.0 & !0b11) | (data_bits as u8))
    }

    pub const fn set_stop_bits(self, stop_bits: StopBits) -> Self {
        Self((self.0 & !0b100) | ((stop_bits as u8) << 2))
    }

    pub const fn set_parity(self, parity: Parity) -> Self {
        Self((self.0 & !0b111000) | ((parity as u8) << 3))
    }

    pub const fn set_break(self, enable_break: bool) -> Self {
        Self((self.0 & !0b1000000) | ((enable_break as u8) << 6))
    }

    pub const fn set_dlab(self, enable_dlab: bool) -> Self {
        Self((self.0 & !0b10000000) | ((enable_dlab as u8) << 7))
    }

    pub const fn data_bits(self) -> DataBits {
        match self.0 & 0b11 {
            0 => DataBits::Bits5,
            1 => DataBits::Bits6,
            2 => DataBits::Bits7,
            3 => DataBits::Bits8,
            _ => unreachable!(),
        }
    }

    pub const fn stop_bits(self) -> StopBits {
        match (self.0 >> 1) & 0b1 {
            0 => StopBits::OneBit,
            1 => StopBits::OneAndHalfBits,
            _ => unreachable!(),
        }
    }

    pub const fn parity(self) -> Parity {
        match (self.0 >> 3) & 0b111 {
            0 | 2 | 4 | 6 => Parity::Disabled,
            1 => Parity::Odd,
            3 => Parity::Even,
            5 => Parity::Forced0,
            7 => Parity::Forced1,
            _ => unreachable!(),
        }
    }

    pub const fn break_bit(self) -> bool {
        (self.0 >> 6) & 1 == 1
    }

    pub const fn dlab_bit(self) -> bool {
        (self.0 >> 7) & 1 == 1
    }
}

impl fmt::Debug for LineControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("LineControl");

        debug_struct.field("data_bits", &self.data_bits());
        debug_struct.field("stop_bits", &self.stop_bits());
        debug_struct.field("parity", &self.parity());
        debug_struct.field("break_bit", &self.break_bit());
        debug_struct.field("dlab_bit", &self.dlab_bit());

        debug_struct.finish()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DataBits {
    Bits5 = 0,
    Bits6 = 1,
    Bits7 = 2,
    Bits8 = 3,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum StopBits {
    OneBit = 0,
    OneAndHalfBits = 1,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Parity {
    Disabled = 0,
    Odd = 1,
    Even = 3,
    Forced1 = 5,
    Forced0 = 7,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct LineStatus(u8);

impl LineStatus {
    pub const fn data_ready(self) -> bool {
        self.0 & 0b1 == 0b1
    }

    pub const fn overrun_error(self) -> bool {
        (self.0 >> 1) & 0b1 == 0b1
    }

    pub const fn parity_error(self) -> bool {
        (self.0 >> 2) & 0b1 == 0b1
    }

    pub const fn framing_error(self) -> bool {
        (self.0 >> 3) & 0b1 == 0b1
    }

    pub const fn break_indicator(self) -> bool {
        (self.0 >> 4) & 0b1 == 0b1
    }

    pub const fn output_empty(self) -> bool {
        (self.0 >> 5) & 0b1 == 0b1
    }

    pub const fn transmitter_empty(self) -> bool {
        (self.0 >> 6) & 0b1 == 0b1
    }

    pub const fn fifo_error(self) -> bool {
        (self.0 >> 7) & 0b1 == 0b1
    }

    pub const fn error_set(self) -> bool {
        self.overrun_error() || self.parity_error() || self.framing_error() || self.fifo_error()
    }
}

fn outb(port: u16, byte: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") byte,
            options(nomem, nostack, preserves_flags)
        );
    }
}

fn inb(port: u16) -> u8 {
    let byte: u8;

    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") byte,
            options(nomem, nostack, preserves_flags)
        );
    }

    byte
}
