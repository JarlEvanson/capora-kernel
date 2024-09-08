//! Module controlling definitions and interfaces to interact with basic system structures.

pub mod gdt;
pub mod idt;

/// The privilege level associated with an item.
pub enum PrivilegeLevel {
    /// Ring 0 is the most privileged ring, used by critical system-software components that
    /// require direct access to, and control over, all processor and system resources.
    Ring0 = 0,
    /// Ring 1 is typically not used anymore, and its privilege is controlled by the operating
    /// system.
    Ring1 = 1,
    /// Ring 2 is typically not used anymore, and its privilege is controlled by the operating
    /// system.
    Ring2 = 2,
    /// Ring 3 is the less privileged ring, used by application software.
    Ring3 = 3,
}
