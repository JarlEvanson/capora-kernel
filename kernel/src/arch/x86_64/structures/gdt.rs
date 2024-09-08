//! Module controlling interaction with the Global Descriptor Table.

use crate::arch::x86_64::structures::PrivilegeLevel;

/// Selects a GDT segment to use.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    /// Selector for the NULL segment.
    pub const NULL: Self = Self::new(0, PrivilegeLevel::Ring0);

    /// Creates a new [`SegmentSelector`] that indicates usage of the segment at `index`, with
    /// requested [`PrivilegeLevel`].
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> Self {
        Self(index << 3 | rpl as u16)
    }

    /// Returns the index of the segment associated with this [`SegmentSelector`].
    pub const fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// The requested [`PrivilegeLevel`] associated with this [`SegmentSelector`].
    pub const fn privilege_level(&self) -> PrivilegeLevel {
        match self.0 & 0b11 {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => unreachable!(),
        }
    }

    /// Sets the requested [`PrivilegeLevel`] associated with this [`SegmentSelector`].
    pub fn set_privilege_level(&mut self, level: PrivilegeLevel) {
        self.0 = self.0 & 0xFFF8 | level as u16
    }
}
