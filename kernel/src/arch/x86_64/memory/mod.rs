//! Module defining basic memory related items.

use core::fmt;

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    pub const MAX_BITS: u8 = 52;
    const ADDRESS_MASK: u64 = (1 << Self::MAX_BITS) - 1;

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(address: u64) -> Option<Self> {
        if address & Self::ADDRESS_MASK != address {
            return None;
        }

        Some(Self(address))
    }

    pub const fn value(&self) -> u64 {
        self.0
    }

    pub const fn frame_offset(&self) -> u64 {
        self.0 % Frame::FRAME_SIZE
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysicalAddress")
            .field(&(self.0 as *const u8))
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(u64);

impl Frame {
    pub const FRAME_SIZE: u64 = 4096;

    pub const fn containing_address(address: PhysicalAddress) -> Self {
        Self(address.value() / Self::FRAME_SIZE)
    }

    pub const fn number(&self) -> u64 {
        self.0
    }

    pub const fn base_address(&self) -> PhysicalAddress {
        PhysicalAddress(self.0 * Self::FRAME_SIZE)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameRange {
    frame: Frame,
    size: u64,
}

impl FrameRange {
    pub const fn inclusive_range(start: Frame, end: Frame) -> Self {
        let size = if end.number() < start.number() {
            0
        } else {
            end.number() - start.number() + 1
        };

        Self { frame: start, size }
    }

    pub const fn start(&self) -> Frame {
        self.frame
    }

    pub const fn start_address(&self) -> PhysicalAddress {
        self.frame.base_address()
    }

    pub const fn size_in_frames(&self) -> u64 {
        self.size
    }

    pub const fn size_in_bytes(&self) -> u64 {
        self.size * Frame::FRAME_SIZE
    }

    pub const fn contains_address(&self, address: PhysicalAddress) -> bool {
        self.start().number() <= Frame::containing_address(address).number()
            && Frame::containing_address(address).number()
                < self.start().number() + self.size_in_frames()
    }

    pub const fn offset_of_address(&self, address: PhysicalAddress) -> Option<u64> {
        if !self.contains_address(address) {
            return None;
        }

        Some(address.value() - self.start_address().value())
    }

    pub const fn address_at_offset(&self, offset: u64) -> Option<PhysicalAddress> {
        if !(offset < self.size_in_bytes()) {
            return None;
        }

        Some(PhysicalAddress(self.start_address().value() + offset))
    }

    pub const fn contains_range(&self, other: &FrameRange) -> bool {
        self.start().number() <= other.start().number()
            && other.start().number() + other.size_in_frames()
                < self.start().number() + self.size_in_frames()
    }

    pub const fn overlaps(&self, other: &FrameRange) -> bool {
        self.start().number() < other.start().number() + other.size_in_frames()
            && other.start().number() < self.start().number() + self.size_in_frames()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub const MAX_BITS: u8 = 48;
    const START_GAP: usize = 0x0000_8000_0000_0000;
    const END_GAP: usize = 0xFFFF_7FFF_FFFF_FFFF;

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(address: usize) -> Option<Self> {
        let upper17 = address & !0x0000_7FFF_FFFF_FFFF;
        if !(upper17 == 0 || upper17 == !0x0000_7FFF_FFFF_FFFF) {
            return None;
        }

        Some(VirtualAddress(address))
    }

    pub const fn value(&self) -> usize {
        self.0
    }

    pub const fn page_offset(&self) -> usize {
        self.0 % Page::PAGE_SIZE
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysicalAddress")
            .field(&(self.0 as *const u8))
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(usize);

impl Page {
    pub const PAGE_SIZE: usize = 4096;

    pub const fn containing_address(address: VirtualAddress) -> Self {
        Self(address.value() / Self::PAGE_SIZE)
    }

    pub const fn number(&self) -> usize {
        self.0
    }

    pub const fn base_address(&self) -> VirtualAddress {
        VirtualAddress(self.0 * Self::PAGE_SIZE)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageRange {
    page: Page,
    size: usize,
}

impl PageRange {
    pub const fn inclusive_range(start: Page, end: Page) -> Option<Self> {
        if start.base_address().value() <= VirtualAddress::END_GAP
            && end.base_address().value() >= VirtualAddress::START_GAP
        {
            return None;
        }

        let size = if end.number() < start.number() {
            0
        } else {
            end.number() - start.number() + 1
        };

        Some(Self { page: start, size })
    }

    pub const fn start(&self) -> Page {
        self.page
    }

    pub const fn start_address(&self) -> VirtualAddress {
        self.page.base_address()
    }

    pub const fn size_in_pages(&self) -> usize {
        self.size
    }

    pub const fn size_in_bytes(&self) -> usize {
        self.size * Page::PAGE_SIZE
    }

    pub const fn contains_address(&self, address: VirtualAddress) -> bool {
        self.start().number() <= Page::containing_address(address).number()
            && Page::containing_address(address).number()
                < self.start().number() + self.size_in_pages()
    }

    pub const fn offset_of_address(&self, address: VirtualAddress) -> Option<usize> {
        if !self.contains_address(address) {
            return None;
        }

        Some(address.value() - self.start_address().value())
    }

    pub const fn address_at_offset(&self, offset: usize) -> Option<VirtualAddress> {
        if !(offset < self.size_in_bytes()) {
            return None;
        }

        Some(VirtualAddress(self.start_address().value() + offset))
    }

    pub const fn contains_range(&self, other: &PageRange) -> bool {
        self.start().number() <= other.start().number()
            && other.start().number() + other.size_in_pages()
                < self.start().number() + self.size_in_pages()
    }

    pub const fn overlaps(&self, other: &PageRange) -> bool {
        self.start().number() < other.start().number() + other.size_in_pages()
            && other.start().number() < self.start().number() + self.size_in_pages()
    }
}
