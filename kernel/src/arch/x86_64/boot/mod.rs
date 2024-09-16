//! Module controlling booting for the kernel on `x86_64`, parsing bootloader structures and
//! transferring to [`kmain`].

use core::mem;

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

    let program_headers = get_phdrs();
    for (index, program_header) in program_headers.iter().enumerate() {
        #[cfg(feature = "logging")]
        log::trace!("Program Header {index}: {:?}", program_header);
    }

    kmain()
}

pub fn get_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
        #[link_name = "phdrs_start"]
        static PHDRS_START: core::ffi::c_void;
        #[link_name = "phdrs_end"]
        static PHDRS_END: core::ffi::c_void;
    }

    let start_ptr = core::ptr::addr_of!(PHDRS_START).cast::<u8>();
    let end_ptr = core::ptr::addr_of!(PHDRS_END).cast::<u8>();

    let size: usize = unsafe { end_ptr.offset_from(start_ptr) }
        .try_into()
        .unwrap();

    let phdrs = unsafe {
        core::slice::from_raw_parts(
            start_ptr.cast::<ProgramHeader>(),
            size / mem::size_of::<ProgramHeader>(),
        )
    };

    #[cfg(feature = "logging")]
    {
        log::trace!("Program headers start: {start_ptr:p}");
        log::trace!("Program headers end: {end_ptr:p}");
        log::trace!("Program headers byte count: {size:#X}");
        log::trace!("Program headers count: {}", phdrs.len());
    }

    phdrs
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramHeader {
    slice: [u8; 56],
}

impl ProgramHeader {
    pub fn segment_type(&self) -> u32 {
        let slice = *self.slice[..4].first_chunk::<4>().unwrap();
        u32::from_ne_bytes(slice)
    }

    pub fn flags(&self) -> u32 {
        let slice = *self.slice[4..8].first_chunk::<4>().unwrap();
        u32::from_ne_bytes(slice)
    }

    pub fn offset(&self) -> u64 {
        let slice = *self.slice[8..16].first_chunk::<8>().unwrap();
        u64::from_ne_bytes(slice)
    }

    pub fn virtual_address(&self) -> u64 {
        let slice = *self.slice[16..24].first_chunk::<8>().unwrap();
        u64::from_ne_bytes(slice)
    }

    pub fn memory_size(&self) -> u64 {
        let slice = *self.slice[40..48].first_chunk::<8>().unwrap();
        u64::from_ne_bytes(slice)
    }
}

impl core::fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_struct = f.debug_struct("ProgramHeader");

        debug_struct.field("segment_type", &self.segment_type());
        debug_struct.field("flags", &self.flags());
        debug_struct.field("offset", &self.offset());
        debug_struct.field("virtual_address", &self.virtual_address());
        debug_struct.field("memory_size", &self.memory_size());

        debug_struct.finish()
    }
}

pub fn setup_idt() {
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    idt.double_fault.set_handler_fn(double_fault_handler);

    unsafe { load_idt(idt) }
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, code: u64) -> ! {
    loop {}
}
