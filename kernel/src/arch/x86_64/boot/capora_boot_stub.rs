//! Module controlling booting using `capora-boot-api`.

use boot_api::{BootloaderRequest, BootloaderResponse};

use crate::arch::x86_64::boot::{karchmain, BootloaderMemoryMapIterator, FrameAllocator};

#[used]
#[link_section = ".bootloader_request"]
static mut BOOTLOADER_REQUEST: BootloaderRequest = BootloaderRequest {
    signature: boot_api::SIGNATURE,
    api_version: boot_api::API_VERSION,
};

/// The entry point when booting using `capora-boot-api` protocol.
#[export_name = "_start"]
pub unsafe extern "C" fn kbootmain(response: *const BootloaderResponse) -> ! {
    #[cfg(feature = "logging")]
    crate::logging::init_logging();

    let response = unsafe { &*response };
    let memory_map = unsafe {
        core::slice::from_raw_parts(response.memory_map_entries, response.memory_map_entry_count)
    };

    let frame_allocator =
        FrameAllocator::new(BootloaderMemoryMapIterator::Capora(memory_map.iter()));

    karchmain(
        response.kernel_virtual_address.cast::<u8>(),
        frame_allocator,
    )
}
