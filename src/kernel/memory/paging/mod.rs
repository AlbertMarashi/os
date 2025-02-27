//! # Page System Implementation
//!
//! This module provides the implementation of our kernel's page system, which manages
//! virtual memory through RISC-V Sv39 paging. The system handles page allocation,
//! page table management, and virtual-to-physical address translation.
//!
//! The page system uses a linked list of free pages and maintains a root page table
//! that maps virtual addresses to physical memory locations. It supports standard
//! RISC-V page table entries (PTEs) with configurable access permissions and attributes.

pub const PAGE_SIZE: u64 = 4096;

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
    /// User read-write access
    pub const USER_RW: u64 = READ_WRITE | USER;
    /// User read-execute access
    pub const USER_RX: u64 = READ_EXECUTE | USER;
}

extern "Rust" {
    static _heap_start: ();
}

impl PageSystem {
    /// ## Create a new [PageSystem]
    ///
    /// Must be initialized with [PageSystem::init] before use
    pub(crate) fn new() -> PageSystem {
        PageSystem {
            root_page_table: core::ptr::null_mut(),
            next_highest_page: core::ptr::addr_of!(_heap_start) as *mut (),
            next_free_page: core::ptr::null_mut(),
        }
    }

    /// ## Initialize the [PageSystem]
    ///
    /// Must be called before using the [PageSystem]
    ///
    /// Creates the root page table for the RISC-V paging system
    pub(crate) fn init(&mut self) {
        // Allocate and initialize the root page table
        self.root_page_table = self.alloc_zeroed_page() as *mut PageTable;
    }

    /// ## Allocate a Page
    /// Allocates a page from the page system.
    /// Returns a mutable pointer to the page.
    ///
    /// ## Safety
    /// This page will not be accessible by code running in lower rings
    /// without allocating a page entry for it within the page system.
    ///
    /// This is merely popping a page from the free list or allocating a new page.
    ///
    /// ```txt
    /// if the next_free_page is null
    ///     let page = next_highest_page
    ///     set next_highest_page to next_highest_page + PAGE_SIZE
    ///     return page
    /// else
    ///     let page = next_free_page
    ///     update the next_free_page to the page.next or null
    ///     return page
    /// ```
    pub(crate) fn alloc_page(&mut self) -> *mut () {
        if self.next_free_page.is_null() {
            let page = self.next_highest_page;
            self.next_highest_page = unsafe { self.next_highest_page.offset(PAGE_SIZE as isize) };
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

    /// ## Allocate a zeroed page
    /// Call the [PageSystem] [PageSystem::alloc_page] and then zeros the page.
    ///
    /// Returns a mutable pointer to the physical page address.
    pub(crate) fn alloc_zeroed_page(&mut self) -> *mut () {
        let page = self.alloc_page();
        unsafe {
            // clear the page to 0
            core::ptr::write_bytes(page, 0, PAGE_SIZE as usize);
        }
        page
    }

    // Helper methods for page table manipulation
    pub(crate) fn get_table_indices(&self, virt_addr: u64) -> [u64; 3] {
        [
            (virt_addr >> 12) & 0x1FF, // Level 0 index
            (virt_addr >> 21) & 0x1FF, // Level 1 index
            (virt_addr >> 30) & 0x1FF, // Level 2 index
        ]
    }

    /// Check if an address is page-aligned
    pub(crate) fn is_page_aligned(&self, addr: u64) -> bool {
        addr % PAGE_SIZE == 0
    }

    /// Find or create a page table at the given level
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

    // Unmap a virtual address
    pub(crate) fn unmap(&mut self, virt_addr: u64) -> Result<(), &'static str> {
        if self.root_page_table.is_null() {
            return Err("Root page table not initialized");
        }

        let indices = self.get_table_indices(virt_addr);

        // Find the leaf entry
        let mut table = self.root_page_table;
        for &idx in indices[1..].iter().rev() {
            let entry = unsafe { &(*table).entries[idx as usize] };
            if !entry.is_valid() {
                return Err("Address not mapped");
            }

            table = (entry.get_ppn() << 12) as *mut PageTable;
        }

        // Clear the leaf entry
        let entry = unsafe { &mut (*table).entries[indices[0] as usize] };
        if !entry.is_valid() {
            return Err("Address not mapped");
        }

        entry.clear(); // Clear the entry completely

        Ok(())
    }

    /// Maps a virtual address to a physical address
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address to map
    /// * `phys_addr` - Physical address to map to
    /// * `flags` - Flags for the mapping (permissions)
    pub(crate) fn map_page(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        // Verify page alignment
        if virt_addr % PAGE_SIZE != 0 || phys_addr % PAGE_SIZE != 0 {
            return Err("Addresses must be page-aligned");
        }

        let indices = self.get_table_indices(virt_addr);
        let mut table = self.root_page_table;

        // Navigate through the higher levels (level 2, level 1)
        for &idx in indices[1..].iter().rev() {
            table = self.get_or_create_table(table, idx);
        }

        // Set the final (level 0) entry
        let entry = unsafe { &mut (*table).entries[indices[0] as usize] };

        entry.set_address(phys_addr as u64, flags);

        Ok(())
    }

    /// Maps a range of virtual addresses to physical addresses with specified offset
    ///
    /// # Arguments
    /// * `virt_start` - Start of virtual address range
    /// * `virt_end` - End of virtual address range
    /// * `phys_start` - Start of physical address range
    /// * `flags` - Flags for the mapping
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err` with a message otherwise
    pub(crate) fn map_range(
        &mut self,
        virt_start: u64,
        virt_end: u64,
        phys_start: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        let virt_start_aligned = virt_start & !(PAGE_SIZE - 1);
        let virt_end_aligned = (virt_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let phys_start_aligned = phys_start & !(PAGE_SIZE - 1);

        // Calculate the offset between virtual and physical addresses
        let offset = phys_start_aligned.wrapping_sub(virt_start_aligned);

        let mut virt_addr = virt_start_aligned;
        while virt_addr < virt_end_aligned {
            let phys_addr = virt_addr.wrapping_add(offset);
            self.map_page(virt_addr, phys_addr, flags)?;
            virt_addr += PAGE_SIZE;
        }

        Ok(())
    }

    // Translate a virtual address to a physical address
    pub(crate) fn translate(&self, virt_addr: u64) -> Option<u64> {
        if self.root_page_table.is_null() {
            return None;
        }

        let indices = self.get_table_indices(virt_addr);
        let page_offset = virt_addr & 0xFFF;

        let mut table = self.root_page_table;

        // Walk through the page tables
        for &idx in indices[1..].iter().rev() {
            let entry = unsafe { &(*table).entries[idx as usize] };
            if !entry.is_valid() {
                return None;
            }

            table = (entry.get_ppn() << 12) as *mut PageTable;
        }

        // Get the leaf entry
        let entry = unsafe { &(*table).entries[indices[0] as usize] };
        if !entry.is_valid() {
            return None;
        }

        // Return the physical address
        Some(((entry.get_ppn() << 12) as u64) | page_offset)
    }

    /// Activates the page table by writing to the SATP register
    ///
    /// This enables the memory management unit (MMU) to use this page table
    /// for virtual address translation.
    ///
    /// # Safety
    ///
    /// This function modifies how the CPU interprets memory addresses. The caller
    /// must ensure that essential memory (code, stack) is properly mapped.
    pub(crate) unsafe fn activate(&self) {
        if self.root_page_table.is_null() {
            return;
        }

        // Set SATP register for Sv39 paging
        // 8 << 60 = Sv39 mode
        let satp_value = (8u64 << 60) | ((self.root_page_table as usize >> 12) as u64);

        core::arch::asm!("csrw satp, {}", in(reg) satp_value);

        // Flush TLB
        core::arch::asm!("sfence.vma");
    }
}

impl PageTableEntry {
    /// Creates a new empty page table entry
    pub(crate) fn new() -> Self {
        PageTableEntry(0)
    }

    /// Checks if the entry is valid (V=1)
    pub(crate) fn is_valid(&self) -> bool {
        self.0 & flags::VALID != 0
    }

    /// Checks if this is a leaf entry (valid and has R, W, or X permissions)
    pub(crate) fn is_leaf(&self) -> bool {
        self.is_valid() && (self.0 & (flags::READABLE | flags::WRITABLE | flags::EXECUTABLE) != 0)
    }

    /// Sets flags in the entry
    ///
    /// # Arguments
    ///
    /// * `flags` - Flags to set
    pub(crate) fn set_flags(&mut self, flags: u64) {
        self.0 |= flags;
    }

    /// Clears flags in the entry
    ///
    /// # Arguments
    ///
    /// * `flags` - Flags to clear
    pub(crate) fn clear_flags(&mut self, flags: u64) {
        self.0 &= !flags;
    }

    /// Checks if all specified flags are set
    ///
    /// # Arguments
    ///
    /// * `flags` - Flags to check
    ///
    /// # Returns
    ///
    /// `true` if all specified flags are set, `false` otherwise
    pub(crate) fn has_flags(&self, flags: u64) -> bool {
        (self.0 & flags) == flags
    }

    /// Sets the physical page number (PPN)
    ///
    /// # Arguments
    ///
    /// * `ppn` - Physical page number
    pub(crate) fn set_ppn(&mut self, ppn: u64) {
        // Clear the PPN bits first (bits 10-53), preserve flags
        self.0 = (self.0 & 0x3FF) | (ppn << 10);
    }

    /// Gets the physical page number (PPN)
    ///
    /// # Returns
    ///
    /// The physical page number
    pub(crate) fn get_ppn(&self) -> u64 {
        (self.0 >> 10) & 0x0000_FFFF_FFFF
    }

    /// Sets both the physical address and flags in one operation
    ///
    /// # Arguments
    ///
    /// * `addr` - Physical address (must be page-aligned)
    /// * `flags` - Flags to set
    pub(crate) fn set_address(&mut self, addr: u64, flags: u64) {
        let ppn = (addr >> 12) & 0x0000_FFFF_FFFF;
        self.0 = (ppn << 10) | flags;
    }

    /// Completely clears the entry
    pub(crate) fn clear(&mut self) {
        self.0 = 0;
    }
}
