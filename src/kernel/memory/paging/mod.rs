//! # Page System Implementation
//!
//! This module provides the implementation of our kernel's page system, which manages
//! virtual memory through RISC-V Sv39 paging. The system handles page allocation,
//! page table management, and virtual-to-physical address translation.
//!
//! The page system supports multiple page sizes (4KB and 2MB) for efficient memory mapping.

/// Standard 4KB page size
pub const PAGE_SIZE_4K: u64 = 4096;
/// Megapage (2MB) size for more efficient mapping of large memory regions
pub const PAGE_SIZE_2M: u64 = 2 * 1024 * 1024;
/// Gigapage (1GB) size (reserved for future use)
pub const _PAGE_SIZE_1G: u64 = 1024 * 1024 * 1024;

#[repr(transparent)]
pub(crate) struct PageTable {
    entries: [PageTableEntry; 512],
}

#[derive(Debug)]
pub(crate) struct PageSystem {
    root_page_table: *mut PageTable,
    next_highest_page: *mut (),
    next_free_page: *mut Page,
}

#[repr(align(4096))]
pub(crate) struct Page {
    next: *mut Page,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(crate) struct PageTableEntry(u64);

/// Page table entry flags for RISC-V Sv39 paging
#[allow(unused)]
pub(crate) mod flags {
    /// Entry is valid (V=1)
    pub const VALID: u64 = 1 << 0;
    /// Page is readable (R=1)
    pub const READABLE: u64 = 1 << 1;
    /// Page is writable (W=1)
    pub const WRITABLE: u64 = 1 << 2;
    /// Page is executable (X=1)
    pub const EXECUTABLE: u64 = 1 << 3;
    /// Accessible in user mode (U=1)
    pub const USER: u64 = 1 << 4;
    /// Global mapping (G=1) - present in all address spaces
    pub const GLOBAL: u64 = 1 << 5;
    /// Page has been accessed (A=1)
    pub const ACCESSED: u64 = 1 << 6;
    /// Page has been modified (D=1)
    pub const DIRTY: u64 = 1 << 7;

    // Common flag combinations
    /// Read-write access (valid + readable + writable)
    pub const READ_WRITE: u64 = VALID | READABLE | WRITABLE;
    /// Read-execute access (valid + readable + executable)
    pub const READ_EXECUTE: u64 = VALID | READABLE | EXECUTABLE;
    /// Full access (valid + readable + writable + executable)
    pub const READ_WRITE_EXECUTE: u64 = VALID | READABLE | WRITABLE | EXECUTABLE;
}

extern "Rust" {
    static _heap_start: ();
}

impl PageSystem {
    /// Creates a new Page System
    ///
    /// Must be initialized with `init()` before use
    pub(crate) fn new() -> PageSystem {
        PageSystem {
            root_page_table: core::ptr::null_mut(),
            next_highest_page: core::ptr::addr_of!(_heap_start) as *mut (),
            next_free_page: core::ptr::null_mut(),
        }
    }

    /// Initializes the Page System
    ///
    /// Creates the root page table for the RISC-V paging system
    pub(crate) fn init(&mut self) {
        // Allocate and initialize the root page table
        self.root_page_table = self.alloc_zeroed_page() as *mut PageTable;
    }

    /// Allocates a new physical page
    ///
    /// Returns a pointer to the allocated page
    pub(crate) fn alloc_page(&mut self) -> *mut () {
        if self.next_free_page.is_null() {
            let page = self.next_highest_page;
            self.next_highest_page =
                unsafe { self.next_highest_page.offset(PAGE_SIZE_4K as isize) };
            page
        } else {
            let page = self.next_free_page;
            self.next_free_page = unsafe {
                match self.next_free_page.as_mut() {
                    Some(p) => p.next,
                    None => core::ptr::null_mut(),
                }
            };
            page as *mut ()
        }
    }

    /// Allocates a zeroed page
    ///
    /// Returns a pointer to the allocated and zeroed page
    pub(crate) fn alloc_zeroed_page(&mut self) -> *mut () {
        let page = self.alloc_page();
        unsafe {
            core::ptr::write_bytes(page, 0, PAGE_SIZE_4K as usize);
        }
        page
    }

    /// Gets the indices for the different page table levels
    ///
    /// Returns [level0_idx, level1_idx, level2_idx]
    pub(crate) fn get_table_indices(&self, virt_addr: u64) -> [u64; 3] {
        [
            (virt_addr >> 12) & 0x1FF, // Level 0 index
            (virt_addr >> 21) & 0x1FF, // Level 1 index
            (virt_addr >> 30) & 0x1FF, // Level 2 index
        ]
    }

    /// Gets or creates a page table at the given level
    ///
    /// Returns a pointer to the page table
    pub(crate) fn get_or_create_table(
        &mut self,
        parent: *mut PageTable,
        index: u64,
    ) -> *mut PageTable {
        let entry = unsafe { &mut (*parent).entries[index as usize] };

        if !entry.is_valid() {
            // Need to create a new page table
            let new_table = self.alloc_zeroed_page() as *mut PageTable;

            // Set up the entry to point to the new table
            entry.set_address(new_table as u64, flags::VALID);

            new_table
        } else {
            // Extract the address from the entry
            let ppn = entry.get_ppn();
            (ppn << 12) as *mut PageTable
        }
    }

    /// Maps a virtual address to a physical address with the specified page size and flags
    ///
    /// # Arguments
    ///
    /// * `virt_addr` - Virtual address to map (must be aligned to page_size)
    /// * `phys_addr` - Physical address to map to (must be aligned to page_size)
    /// * `page_size` - Size of the page (PAGE_SIZE_4K or PAGE_SIZE_2M)
    /// * `flags` - Page table entry flags
    ///
    /// # Returns
    ///
    /// * `Ok(())` if mapping was successful
    /// * `Err` with an error message if mapping failed
    pub(crate) fn map(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        page_size: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        // Verify page alignment based on page size
        if virt_addr % page_size != 0 || phys_addr % page_size != 0 {
            return Err("Addresses must be aligned to the specified page size");
        }

        match page_size {
            PAGE_SIZE_4K => {
                // 4KB page mapping
                let indices = self.get_table_indices(virt_addr);
                let mut table = self.root_page_table;

                // Navigate through the higher levels (level 2, level 1)
                for &idx in indices[1..].iter().rev() {
                    table = self.get_or_create_table(table, idx);
                }

                // Set the final (level 0) entry
                let entry = unsafe { &mut (*table).entries[indices[0] as usize] };
                entry.set_address(phys_addr, flags);

                Ok(())
            }
            PAGE_SIZE_2M => {
                // 2MB megapage mapping
                let indices = self.get_table_indices(virt_addr);
                let level2_idx = indices[2];
                let level1_idx = indices[1];

                // Get or create the level 2 table entry
                let level2_entry =
                    unsafe { &mut (*self.root_page_table).entries[level2_idx as usize] };

                let level1_table = if !level2_entry.is_valid() {
                    // Create a new level 1 page table
                    let new_table = self.alloc_zeroed_page() as *mut PageTable;
                    level2_entry.set_address(new_table as u64, flags::VALID);
                    new_table
                } else {
                    // Use existing level 1 page table
                    (level2_entry.get_ppn() << 12) as *mut PageTable
                };

                // Create a leaf entry at level 1 (megapage)
                let level1_entry = unsafe { &mut (*level1_table).entries[level1_idx as usize] };

                // For a megapage, ensure the lowest 9 bits of the PPN are zero
                let mega_ppn = (phys_addr >> 12) & !0x1FF;
                level1_entry.0 = (mega_ppn << 10) | (flags | flags::VALID);

                Ok(())
            }
            _ => Err("Unsupported page size"),
        }
    }

    /// Maps a 4KB page
    ///
    /// Maps a virtual address to a physical address with specified flags
    pub(crate) fn map_page(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        self.map(virt_addr, phys_addr, PAGE_SIZE_4K, flags)
    }

    /// Maps a range of memory using the most efficient page sizes
    ///
    /// Uses 2MB pages where possible, falling back to 4KB pages when needed
    pub(crate) fn map_range_optimized(
        &mut self,
        virt_start: u64,
        virt_end: u64,
        phys_start: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        let mut virt_addr = virt_start;
        let mut phys_addr = phys_start;

        while virt_addr < virt_end {
            // Try to map a 2MB page if alignment permits
            if virt_addr % PAGE_SIZE_2M == 0
                && phys_addr % PAGE_SIZE_2M == 0
                && virt_addr + PAGE_SIZE_2M <= virt_end
            {
                // We can map a 2MB page here
                if let Err(e) = self.map(virt_addr, phys_addr, PAGE_SIZE_2M, flags) {
                    return Err(e);
                }
                virt_addr += PAGE_SIZE_2M;
                phys_addr += PAGE_SIZE_2M;
            } else {
                // Fall back to 4KB page
                if let Err(e) = self.map(virt_addr, phys_addr, PAGE_SIZE_4K, flags) {
                    return Err(e);
                }
                virt_addr += PAGE_SIZE_4K;
                phys_addr += PAGE_SIZE_4K;
            }
        }

        Ok(())
    }

    /// Activates the page table by writing to the SATP register
    ///
    /// # Safety
    ///
    /// This function enables virtual memory translation. The caller
    /// must ensure that essential memory is properly mapped.
    pub(crate) unsafe fn activate(&self) {
        if self.root_page_table.is_null() {
            return;
        }

        // Set SATP register for Sv39 paging (mode 8)
        let satp_value = (8u64 << 60) | ((self.root_page_table as usize >> 12) as u64);
        core::arch::asm!("csrw satp, {}", in(reg) satp_value);

        // Flush TLB
        core::arch::asm!("sfence.vma");
    }
}

impl PageTableEntry {
    /// Checks if the entry is valid (V=1)
    pub(crate) fn is_valid(&self) -> bool {
        self.0 & flags::VALID != 0
    }

    /// Gets the physical page number (PPN)
    pub(crate) fn get_ppn(&self) -> u64 {
        (self.0 >> 10) & 0x0000_FFFF_FFFF
    }

    /// Sets both the physical address and flags
    pub(crate) fn set_address(&mut self, addr: u64, flags: u64) {
        let ppn = (addr >> 12) & 0x0000_FFFF_FFFF;
        self.0 = (ppn << 10) | flags;
    }
}
