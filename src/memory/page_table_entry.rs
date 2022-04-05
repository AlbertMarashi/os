use core::ops::{BitOrAssign, BitAndAssign, BitOr};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(crate) struct PageTableEntry(u64);

pub(crate) struct PPN(pub u64);

pub(crate) fn physical_addr_to_ppn(addr: u64) -> PPN {
    PPN((addr << 8) >> (12 + 8))
}

#[repr(u64)]
pub(crate) enum PageTableEntryFlag {
    Valid = 1 << 0,
    Readable = 1 << 1,
    Writable = 1 << 2,
    Executable = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Accessed = 1 << 6,
    Dirty = 1 << 7,
    // PageTable = PageTableEntryFlag::Readable as u64 | PageTableEntryFlag::Writable as u64 | PageTableEntryFlag::Executable as u64,
    // ReadWrite = PageTableEntryFlag::Readable as u64 | PageTableEntryFlag::Writable as u64,
}

impl BitOrAssign<PageTableEntryFlag> for u64 {
    fn bitor_assign(&mut self, rhs: PageTableEntryFlag) {
        *self |= rhs as u64;
    }
}

impl BitAndAssign<PageTableEntryFlag> for u64 {
    fn bitand_assign(&mut self, rhs: PageTableEntryFlag) {
        *self &= rhs as u64;
    }
}

impl PageTableEntry {
    pub(crate) fn set_flag(&mut self, flag: PageTableEntryFlag, value: bool) {
        if value {
            self.0 |= flag
        } else {
            self.0 &= !(flag as u64)
        }
    }

    pub(crate) fn get_flag(&self, flag: PageTableEntryFlag) -> bool {
        self.0 & flag as u64 != 0
    }

    pub(crate) fn set_ppn(&mut self, ppn: PPN) {
        self.0 = ppn.0 << 10;
    }
}

