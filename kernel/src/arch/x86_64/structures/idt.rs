//! Module controlling interaction with the [`InterruptDescriptorTable`].

use core::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
};

use crate::arch::{
    x86_64::memory::VirtualAddress,
    x86_64::structures::{gdt::SegmentSelector, PrivilegeLevel},
};

/// Table of [`InterruptDescriptor`]s that describe how an interrupt should be handled.
#[repr(C, align(4096))]
pub struct InterruptDescriptorTable {
    /// Indicates the divisor operand for a DIV or IDIV instruction is 0 or that the result cannot
    /// be represented in the number of bits for the destination operand.
    pub divide_error: InterruptDescriptor<HandlerFunc>,
    /// Indicates that one or more of several debug-exceptions conditions has been detected.
    pub debug: InterruptDescriptor<HandlerFunc>,
    /// A non-maskable interrupt has occurred.
    pub non_maskable_interrupt: InterruptDescriptor<HandlerFunc>,
    /// Indicates that a breakpoint instruction was executed, causing a breakpoint trap to be
    /// generated.
    pub breakpoint: InterruptDescriptor<HandlerFunc>,
    /// An overflow trap occurred when an INTO instruction was executed.
    pub overflow: InterruptDescriptor<HandlerFunc>,
    /// Indicates that a BOUND-range-exceeded fault occurred when a BOUND instruction was executed.
    pub bound_range_exceeded: InterruptDescriptor<HandlerFunc>,
    /// Indicates that the processor attempted to execute an invalid or reserved opcode, or an
    /// instruction with illegal arguments.
    pub invalid_opcode: InterruptDescriptor<HandlerFunc>,
    /// The processor executed a x87 FPU floating-point instruction while the EM flag in the
    /// control register CR0 was set, the processor executed a WAIT/FWAIT instruction while the MP
    /// and TS flags of the register CR0 were set, regardless of the setting of the EM flag, or the
    /// processor executed an x87 FPU, MMX, or SSE/SSE2/SSE3 instruction while the TS flag in
    /// control register CR0 was set and the EM flag is clear.
    pub device_not_available: InterruptDescriptor<HandlerFunc>,
    /// Indicates that the processor detected a second exception while calling an exception handler
    /// for a prior exception.
    pub double_fault: InterruptDescriptor<NoReturnHandlerFuncErrorCode>,
    /// Reserved interrupt.
    pub coprocessor_segment_overrun: InterruptDescriptor<HandlerFunc>,
    /// An error related to a TSS occurred.
    pub invalid_tss: InterruptDescriptor<HandlerFuncErrorCode>,
    /// The present flag of a segment or gate descriptor is clear.
    pub segment_not_present: InterruptDescriptor<HandlerFuncErrorCode>,
    /// Either a limit violation was detected during an operation that refers to the SS register, a
    /// not-present stack segment was detected when attempted to switch stack segemnts, or a
    /// canonical violation was detected during an operation that references memory using the stack
    /// pointer.
    pub stack_segment_fault: InterruptDescriptor<HandlerFuncErrorCode>,
    /// The processor detected a class of protection violations that does not trigger another
    /// interrupt.
    pub general_protection_fault: InterruptDescriptor<HandlerFuncErrorCode>,
    /// Indicates, that with paging enabled, the processor detected an error while using the
    /// page-translation mechanism to translate a linear address to a physical address.
    pub page_fault: InterruptDescriptor<HandlerFuncErrorCode>,
    /// Reserved interrupt.
    pub _reserved_1: InterruptDescriptor<HandlerFunc>,
    /// The x87 FPU detected a floating point error.
    pub x87_floating_point_fault: InterruptDescriptor<HandlerFunc>,
    /// The processor detected an unaligned memory operand when alignment checking was enabled.
    pub alignment_check_exception: InterruptDescriptor<HandlerFuncErrorCode>,
    /// Indicates that the processor detected an internal machine or bus error, or that an external
    /// agent detected a bus error.
    ///
    /// This is model specific.
    pub machine_check: InterruptDescriptor<NoReturnHandlerFunc>,
    /// Indicates that the processor detected an SSE/SSE2/SSE3 SIMD floating point exception.
    pub simd_floating_point: InterruptDescriptor<HandlerFunc>,
    /// Indicates that the processor detected an EPT violation in VMX non-root operation.
    pub virtualization: InterruptDescriptor<HandlerFunc>,
    /// Indicates that the processor detected a control flow transfer attempt would have violated
    /// the control flow enforcement technology constraints.
    pub cp_protection_exception: InterruptDescriptor<HandlerFuncErrorCode>,
    /// Reserved interrupts.
    pub _reserved_2: [InterruptDescriptor<HandlerFunc>; 10],

    /// General purpose interrupts.
    pub general_interrupts: [InterruptDescriptor<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    /// Creates a new [`InterruptDescriptorTable`], setting all entries to
    /// [`InterruptDescriptor::MISSING`].
    pub const fn new() -> Self {
        Self {
            divide_error: InterruptDescriptor::MISSING,
            debug: InterruptDescriptor::MISSING,
            non_maskable_interrupt: InterruptDescriptor::MISSING,
            breakpoint: InterruptDescriptor::MISSING,
            overflow: InterruptDescriptor::MISSING,
            bound_range_exceeded: InterruptDescriptor::MISSING,
            invalid_opcode: InterruptDescriptor::MISSING,
            device_not_available: InterruptDescriptor::MISSING,
            double_fault: InterruptDescriptor::MISSING,
            coprocessor_segment_overrun: InterruptDescriptor::MISSING,
            invalid_tss: InterruptDescriptor::MISSING,
            segment_not_present: InterruptDescriptor::MISSING,
            stack_segment_fault: InterruptDescriptor::MISSING,
            general_protection_fault: InterruptDescriptor::MISSING,
            page_fault: InterruptDescriptor::MISSING,
            _reserved_1: InterruptDescriptor::MISSING,
            x87_floating_point_fault: InterruptDescriptor::MISSING,
            alignment_check_exception: InterruptDescriptor::MISSING,
            machine_check: InterruptDescriptor::MISSING,
            simd_floating_point: InterruptDescriptor::MISSING,
            virtualization: InterruptDescriptor::MISSING,
            cp_protection_exception: InterruptDescriptor::MISSING,
            _reserved_2: [InterruptDescriptor::MISSING; 10],
            general_interrupts: [InterruptDescriptor::MISSING; 256 - 32],
        }
    }
}

/// 16-byte structure that identifies the [`VirtualAddress`] of a handler function, as well as
/// other miscellaneous information that determines how an interrupt occurs.
#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InterruptDescriptor<F> {
    low_func_ptr: u16,
    code_segment: SegmentSelector,
    options: InterruptDescriptorOptions,
    mid_func_ptr: u16,
    high_func_ptr: u32,
    _reserved: u32,
    phantom: PhantomData<F>,
}

impl<F> InterruptDescriptor<F> {
    /// An [`InterruptDescriptor`] that descibes a missing handler function.
    pub const MISSING: Self = Self {
        low_func_ptr: 0,
        code_segment: SegmentSelector::NULL,
        options: InterruptDescriptorOptions::MISSING,
        mid_func_ptr: 0,
        high_func_ptr: 0,
        _reserved: 0,
        phantom: PhantomData,
    };

    /// Constructs a new [`InterruptDescriptor`] that points to a handler function located at
    /// `address`, with the CPU using `code_segment` to determine the code segment, and `options`
    /// to determine the behavior of the CPU on interrupt.
    pub const unsafe fn new(
        address: VirtualAddress,
        code_segment: SegmentSelector,
        options: InterruptDescriptorOptions,
    ) -> Self {
        Self {
            low_func_ptr: address.value() as u16,
            code_segment,
            options,
            mid_func_ptr: (address.value() >> 16) as u16,
            high_func_ptr: (address.value() >> 32) as u32,
            _reserved: 0,
            phantom: PhantomData,
        }
    }

    /// The [`VirtualAddress`] of the handler function.
    pub fn func_ptr(&self) -> VirtualAddress {
        VirtualAddress::new(
            self.low_func_ptr as usize
                | ((self.mid_func_ptr as usize) << 16)
                | ((self.high_func_ptr as usize) << 32),
        )
        .unwrap()
    }

    /// Sets the [`SegmentSelector`] which the CPU uses for the code segment when the interrupt
    /// occurs.
    pub unsafe fn set_code_segment(&mut self, segment: SegmentSelector) {
        self.code_segment = segment;
    }

    /// Sets the [`InterruptDescriptorOptions`], which controls the behavior of the interrupt and
    /// interrupt handler.
    pub unsafe fn set_options(&mut self, options: InterruptDescriptorOptions) {
        self.options = options;
    }
}

impl<F: HandlerFuncSupport> InterruptDescriptor<F> {
    /// Sets the address of the handler function to the value of `handler.address()`.
    ///
    /// Also sets the code segment selector to select the segment in index 2 at
    /// [`PrivilegeLevel::Ring0`] as the code segment and the options to indicate that the
    /// interrupt handler is present, should disable interrupts, operate on the same stack, and
    /// handle the interrupt at [`PrivilegeLevel::Ring0`].
    pub fn set_handler_fn(&mut self, handler: F) {
        let address = handler.address().value();

        self.low_func_ptr = address as u16;
        self.mid_func_ptr = (address >> 16) as u16;
        self.high_func_ptr = (address >> 32) as u32;

        self.options = InterruptDescriptorOptions::new(
            true,
            IstSetting::NoSwitch,
            true,
            PrivilegeLevel::Ring0,
        );
        self.code_segment = SegmentSelector::new(2, PrivilegeLevel::Ring0);
    }
}

/// Loads the provided [`InterruptDescriptorTable`].
pub unsafe fn load_idt(table: &'static mut InterruptDescriptorTable) {
    #[repr(C)]
    struct Idtr {
        _unused: MaybeUninit<[u8; 6]>,
        size: u16,
        address: u64,
    }

    let idtr = Idtr {
        _unused: MaybeUninit::uninit(),
        size: (mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
        address: table as *mut InterruptDescriptorTable as u64,
    };

    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idtr.size,
        )
    }
}

/// Various options that control the behavior of the interrupt when it occurs.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InterruptDescriptorOptions(u16);

impl InterruptDescriptorOptions {
    /// An [`InterruptDescriptorOptions`] that describes a missing [`InterruptDescriptor`].
    pub const MISSING: Self = Self::new(false, IstSetting::NoSwitch, true, PrivilegeLevel::Ring0);

    /// Creates a new [`InterruptDescriptorOptions`], which specifies whether the interrupt handler
    /// is present, which stack to switch to when the handler is called, whether interrupts are
    /// disabled for the duration of the interrupt, and privilege_level at which the interrupt
    /// handling occurs.
    pub const fn new(
        present: bool,
        ist: IstSetting,
        disables_interrupts: bool,
        privilege_level: PrivilegeLevel,
    ) -> Self {
        Self(
            (ist as u16)
                | ((disables_interrupts as u16) << 8)
                | (0b111 << 9)
                | ((privilege_level as u16) << 13)
                | ((present as u16) << 15),
        )
    }

    /// Which stack to switch to when this interrupt occurs.
    pub const fn ist(&self) -> IstSetting {
        match self.0 & 0b111 {
            0 => IstSetting::NoSwitch,
            1 => IstSetting::Ist1,
            2 => IstSetting::Ist2,
            3 => IstSetting::Ist3,
            4 => IstSetting::Ist4,
            5 => IstSetting::Ist5,
            6 => IstSetting::Ist6,
            7 => IstSetting::Ist7,
            _ => unreachable!(),
        }
    }

    /// Wether interrupts are disabled when this interrupt occurs.
    pub const fn disables_interrupts(&self) -> bool {
        !(self.0 & (1 << 8) == (1 << 8))
    }

    /// The privilege_level to switch to when this interrupt occurs.
    pub const fn privilege_level(&self) -> PrivilegeLevel {
        match (self.0 >> 13) & 0b11 {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => unreachable!(),
        }
    }

    /// Whether the interrupt handler is present.
    pub const fn present(&self) -> bool {
        self.0 & (1 << 15) == (1 << 15)
    }
}

/// The stack to switch to if when handling the interrupt occurs.
pub enum IstSetting {
    /// Don't switch stacks.
    NoSwitch = 0,
    /// Switch to the 1st stack in the interrupt stack table.
    Ist1 = 1,
    /// Switch to the 2nd stack in the interrupt stack table.
    Ist2 = 2,
    /// Switch to the 3rd stack in the interrupt stack table.
    Ist3 = 3,
    /// Switch to the 4th stack in the interrupt stack table.
    Ist4 = 4,
    /// Switch to the 5th stack in the interrupt stack table.
    Ist5 = 5,
    /// Switch to the 6th stack in the interrupt stack table.
    Ist6 = 6,
    /// Switch to the 7th stack in the interrupt stack table.
    Ist7 = 7,
}

pub trait HandlerFuncSupport {
    fn address(self) -> VirtualAddress;
}

impl HandlerFuncSupport for NoReturnHandlerFunc {
    fn address(self) -> VirtualAddress {
        unsafe { VirtualAddress::new(self as usize).unwrap_unchecked() }
    }
}

impl HandlerFuncSupport for NoReturnHandlerFuncErrorCode {
    fn address(self) -> VirtualAddress {
        unsafe { VirtualAddress::new(self as usize).unwrap_unchecked() }
    }
}

impl HandlerFuncSupport for HandlerFunc {
    fn address(self) -> VirtualAddress {
        unsafe { VirtualAddress::new(self as usize).unwrap_unchecked() }
    }
}

impl HandlerFuncSupport for HandlerFuncErrorCode {
    fn address(self) -> VirtualAddress {
        unsafe { VirtualAddress::new(self as usize).unwrap_unchecked() }
    }
}

type NoReturnHandlerFunc = extern "x86-interrupt" fn(_: InterruptStackFrame) -> !;
type NoReturnHandlerFuncErrorCode =
    extern "x86-interrupt" fn(_: InterruptStackFrame, error_code: u64) -> !;
type HandlerFunc = extern "x86-interrupt" fn(_: InterruptStackFrame);
type HandlerFuncErrorCode = extern "x86-interrupt" fn(_: InterruptStackFrame, error_code: u64);

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    interrupt_pointer: VirtualAddress,
    code_segment: SegmentSelector,
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    stack_segment: SegmentSelector,
}
