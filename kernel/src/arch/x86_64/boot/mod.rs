//! Module controlling booting for the kernel on `x86_64`, parsing bootloader structures and
//! transferring to [`kmain`].

use core::{mem, slice};

use crate::{
    arch::x86_64::{
        memory::{
            Frame, FrameRange, FrameRangeIter, Page, PageRange, PhysicalAddress, VirtualAddress,
        },
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
pub fn karchmain(kernel_address: *const u8, allocator: FrameAllocator) -> ! {
    setup_idt();

    let mut pml4e_index = 512;
    let mut pml3e_index = 512;
    let mut pml2e_index = 512;

    let mut page_table_page_count: usize = 1;
    let mut kernel_backing_frame_count: usize = 0;

    let program_headers = get_phdrs();
    for (index, program_header) in program_headers.iter().enumerate() {
        #[cfg(feature = "logging")]
        log::trace!("Program Header {index}: {:?}", program_header);

        if program_header.segment_type() != 1 {
            continue;
        }

        let page = Page::containing_address(VirtualAddress::new_canonical(
            kernel_address as usize + program_header.virtual_address() as usize,
        ));
        let end_page = Page::containing_address(VirtualAddress::new_canonical(
            (kernel_address as u64
                + program_header.virtual_address()
                + (program_header.memory_size() - 1)) as usize,
        ));
        let page_range = PageRange::inclusive_range(page, end_page).unwrap();

        for page in page_range {
            if page.pml4e_index() != pml4e_index {
                pml4e_index = page.pml4e_index();
                page_table_page_count += 1;

                pml3e_index = 512;
                pml2e_index = 512;
            }
            if page.pml3e_index() != pml3e_index {
                pml3e_index = page.pml3e_index();
                page_table_page_count += 1;

                pml2e_index = 512;
            }
            if page.pml2e_index() != pml2e_index {
                pml2e_index = page.pml2e_index();
                page_table_page_count += 1;
            }
        }
        kernel_backing_frame_count += page_range.size_in_pages();
    }

    #[cfg(feature = "logging")]
    log::trace!("{allocator:#X?}");

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

#[derive(Clone, Debug)]
pub struct FrameAllocator {
    original: BootloaderMemoryMapIterator,
    entries: BootloaderMemoryMapIterator,
    current: FrameRangeIter,
}

impl FrameAllocator {
    fn new(entries: BootloaderMemoryMapIterator) -> FrameAllocator {
        FrameAllocator {
            original: entries.clone(),
            entries,
            current: FrameRangeIter::empty(),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let mut next_frame = self.current.next();
        while next_frame.is_none() {
            self.current = self.entries.next()?.into_iter();
            next_frame = self.current.next();
        }

        next_frame
    }
}

#[derive(Clone, Debug)]
enum BootloaderMemoryMapIterator {
    #[cfg(feature = "capora-boot-api")]
    Capora(slice::Iter<'static, boot_api::MemoryMapEntry>),
    #[cfg(feature = "limine-boot-api")]
    Limine(slice::Iter<'static, &'static limine::MemoryMapEntry>),
}

impl Iterator for BootloaderMemoryMapIterator {
    type Item = FrameRange;

    fn next(&mut self) -> Option<Self::Item> {
        let (base_address, size) = match self {
            #[cfg(feature = "capora-boot-api")]
            Self::Capora(iter) => {
                let mut entry = iter.next()?;
                while entry.kind != boot_api::MemoryMapEntryKind::USABLE {
                    entry = iter.next()?;
                }

                (entry.base, entry.size)
            }
            #[cfg(feature = "limine-boot-api")]
            Self::Limine(iter) => {
                let mut entry = iter.next()?;
                while entry.mem_type != limine::MemoryMapEntryType::USABLE {
                    entry = iter.next()?;
                }

                (entry.base, entry.length)
            }
        };
        if size == 0 {
            return self.next();
        }

        let Some(base_address) = PhysicalAddress::new(base_address) else {
            #[cfg(feature = "logging")]
            log::warn!("Memory map entry outside of valid physical address range");
            return None;
        };

        let Some(end_address) = base_address
            .value()
            .checked_add(size)
            .and_then(|end_address| PhysicalAddress::new(end_address - 1))
        else {
            #[cfg(feature = "logging")]
            log::warn!("Memory map entry outside of valid physical address range");
            return None;
        };
        Some(FrameRange::inclusive_range(
            Frame::containing_address(base_address),
            Frame::containing_address(end_address),
        ))
    }
}
