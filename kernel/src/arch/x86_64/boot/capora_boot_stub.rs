//! Module controlling booting using `capora-boot-api`.

use boot_api::{BootloaderRequest, BootloaderResponse};

use crate::arch::x86_64::boot::karchmain;

#[used]
#[link_section = ".bootloader_request"]
static mut BOOTLOADER_REQUEST: BootloaderRequest = BootloaderRequest {
    signature: boot_api::SIGNATURE,
    api_version: boot_api::API_VERSION,
};

/// The entry point when booting using `capora-boot-api` protocol.
#[export_name = "_start"]
pub unsafe extern "C" fn kbootmain(response: *const BootloaderResponse) -> ! {
    karchmain()
}
