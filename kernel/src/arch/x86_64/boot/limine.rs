//! Module controlling booting using the Limine boot protocol.

use crate::{arch::x86_64::boot::karchmain, cells::ControlledModificationCell};

/// The base revision of the Limine boot protocol that this kernel supports.
pub const LIMINE_BASE_REVISION: u64 = 2;

/// The first Limine magic number.
pub const LIMINE_MAGIC_0: u64 = 0xc7b1dd30df4c8b88;
/// The second Limine magic number.
pub const LIMINE_MAGIC_1: u64 = 0x0a82e883a194f07b;

/// A tag indicating that this executable uses the Limine boot protocol and that it supports
/// [`LIMINE_BASE_REVISION`].
#[used]
#[link_section = ".limine_requests"]
static LIMINE_BASE_REVISION_TAG: ControlledModificationCell<[u64; 3]> =
    ControlledModificationCell::new([0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, LIMINE_BASE_REVISION]);

/// A request to enter at the given function from the bootloader.
#[used]
#[link_section = ".limine_requests"]
static LIMINE_ENTRY_POINT_REQUEST: ControlledModificationCell<Request<EntryPointRequest>> =
    ControlledModificationCell::new(Request::new(EntryPointRequest::new(kbootmain)));

/// A request for the memory map from the bootloader.
#[used]
#[link_section = ".limine_requests"]
static LIMINE_MEMORY_MAP_REQUEST: ControlledModificationCell<Request<MemoryMapRequest>> =
    ControlledModificationCell::new(Request::new(MemoryMapRequest::new()));

/// A request to obtain the virtual and physical address of the kernel.
#[used]
#[link_section = ".limine_requests"]
static LIMINE_KERNEL_ADDRESS_REQUEST: ControlledModificationCell<Request<KernelAddressRequest>> =
    ControlledModificationCell::new(Request::new(KernelAddressRequest::new()));

/// A request to obtain the offset of the higher half memory direct map.
#[used]
#[link_section = ".limine_requests"]
static LIMINE_HIGHER_DIRECT_MAP_REQUEST: ControlledModificationCell<Request<DirectMapRequest>> =
    ControlledModificationCell::new(Request::new(DirectMapRequest::new()));

/// The entry point when using the Limine boot protocol.
#[cfg_attr(not(feature = "capora-boot-api"), export_name = "_start")]
pub unsafe extern "C" fn kbootmain() -> ! {
    #[cfg(feature = "logging")]
    crate::logging::init_logging();

    if LIMINE_BASE_REVISION_TAG.get()[2] == LIMINE_BASE_REVISION {
        loop {}
    }

    karchmain()
}

/// The base structure of a [`LimineRequest`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Request<T: LimineRequest> {
    id: [u64; 4],
    revision: u64,
    response: *mut Response<T::Response>,
    body: T,
}

unsafe impl<T: LimineRequest + Send> Send for Request<T> {}

impl<T: LimineRequest> Request<T> {
    pub const fn new(body: T) -> Self {
        Self {
            id: T::ID,
            revision: T::REVISION,
            response: core::ptr::null_mut(),
            body,
        }
    }

    /// Returns [`&Response<T::Response>`] if the request is supported, otherwise, if the
    /// [`LimineResponse`] is unsupported or was not successfully processed, this returns [`None`].
    pub fn response(&self) -> Option<&Response<T::Response>> {
        unsafe { self.response.as_ref() }
    }
}

/// The base structure of a [`LimineResponse`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Response<T: LimineResponse> {
    revision: u64,
    body: T,
}

impl<T: LimineResponse> Response<T> {
    pub fn body(&self) -> Option<&T> {
        if !self.is_supported() {
            return None;
        }

        Some(&self.body)
    }

    pub fn is_supported(&self) -> bool {
        self.revision() >= T::REVISION
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPointRequest {
    entry_point: unsafe extern "C" fn() -> !,
}

impl EntryPointRequest {
    pub const fn new(entry_point: unsafe extern "C" fn() -> !) -> Self {
        Self { entry_point }
    }
}

impl LimineRequest for EntryPointRequest {
    const ID: [u64; 4] = [
        LIMINE_MAGIC_0,
        LIMINE_MAGIC_1,
        0x13d86c035a1cd3e1,
        0x2b0caa89d8f3026a,
    ];
    const REVISION: u64 = 0;
    type Response = EntryPointResponse;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPointResponse();

impl LimineResponse for EntryPointResponse {
    const REVISION: u64 = 0;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapRequest();

impl MemoryMapRequest {
    pub const fn new() -> Self {
        Self()
    }
}

impl LimineRequest for MemoryMapRequest {
    const ID: [u64; 4] = [
        LIMINE_MAGIC_0,
        LIMINE_MAGIC_1,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ];
    const REVISION: u64 = 0;
    type Response = MemoryMapResponse;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapResponse {
    entry_count: u64,
    entries: *mut *mut MemoryMapEntry,
}

impl LimineResponse for MemoryMapResponse {
    const REVISION: u64 = 0;
}

impl MemoryMapResponse {
    pub fn as_slice(&self) -> &[&MemoryMapEntry] {
        assert!(!self.entries.is_null());
        let slice = unsafe { core::slice::from_raw_parts(self.entries, self.entry_count as usize) };
        for entry in slice {
            assert!(!entry.is_null());
        }

        unsafe {
            core::slice::from_raw_parts(
                self.entries.cast::<&MemoryMapEntry>(),
                self.entry_count as usize,
            )
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntry {
    base: u64,
    length: u64,
    mem_type: MemoryMapEntryType,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntryType(u64);

impl MemoryMapEntryType {
    pub const USABLE: Self = Self(0);
    pub const RESERVED: Self = Self(1);
    pub const ACPI_RECLAIMABLE: Self = Self(2);
    pub const ACPI_NVS: Self = Self(3);
    pub const BAD_MEMORY: Self = Self(4);
    pub const BOOTLOADER_RECLAIMABLE: Self = Self(5);
    pub const KERNEL_AND_MODULES: Self = Self(6);
    pub const FRAMEBUFFER: Self = Self(7);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelAddressRequest();

impl KernelAddressRequest {
    pub const fn new() -> Self {
        Self()
    }
}

impl LimineRequest for KernelAddressRequest {
    const ID: [u64; 4] = [
        LIMINE_MAGIC_0,
        LIMINE_MAGIC_1,
        0x71ba76863cc55f63,
        0xb2644a48c516a487,
    ];
    const REVISION: u64 = 0;
    type Response = KernelAddressResponse;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelAddressResponse {
    physical_base: u64,
    virtual_base: u64,
}

impl LimineResponse for KernelAddressResponse {
    const REVISION: u64 = 0;
}

pub trait LimineRequest {
    /// The ID used by the [`LimineProtocol`] request.
    const ID: [u64; 4];
    /// The revision of the request that the kernel provides.
    const REVISION: u64;

    type Response: LimineResponse;
}

pub trait LimineResponse {
    /// The revision of the response that the kernel supports.
    const REVISION: u64;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectMapRequest();

impl DirectMapRequest {
    pub const fn new() -> Self {
        Self()
    }
}

impl LimineRequest for DirectMapRequest {
    const ID: [u64; 4] = [
        LIMINE_MAGIC_0,
        LIMINE_MAGIC_1,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ];
    const REVISION: u64 = 0;
    type Response = DirectMapResponse;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectMapResponse {
    offset: u64,
}

impl LimineResponse for DirectMapResponse {
    const REVISION: u64 = 0;
}
