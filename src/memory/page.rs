
use super::page_table_entry::PageTableEntry;

const LEVELS: u8 = 4;
const PTESIZE: u8 = 8;
pub const PAGE_SIZE: usize = 4096;

#[repr(transparent)]
pub(crate) struct PageTable {
    entries: [PageTableEntry; 512]
}


#[derive(Debug)]
pub(crate) struct PageSystem {
    root_page_table: *mut PageTable,
    next_highest_page: *mut (),
    next_free_page: *mut Page
}

#[repr(align(4096))]
pub(crate) struct Page {
    next: *mut Page,
}

extern "Rust" {
    static _heap_start: ();
}

impl PageSystem {
    pub(crate) fn new() -> PageSystem {
        PageSystem {
            root_page_table: core::ptr::null_mut(),
            next_highest_page: unsafe { core::ptr::addr_of!(_heap_start) as *mut () },
            next_free_page: core::ptr::null_mut()
        }
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
    fn alloc_page(&mut self) -> *mut () {
        if self.next_free_page.is_null() {
            let page = self.next_highest_page;
            self.next_highest_page = unsafe { self.next_highest_page.offset(PAGE_SIZE as isize) };
            page
        } else {
            let page = self.next_free_page;
            self.next_free_page = unsafe {
                match self.next_free_page.as_mut() {
                    Some(p) => p.next,
                    None => core::ptr::null_mut()
                }
            };
            page as *mut ()
        }
    }

    /// ## Allocate a zeroed page
    /// Call the [PageSystem] [PageSystem::alloc_page] and then zeros the page.
    ///
    /// Returns a mutable pointer to the physical page address.
    fn alloc_zeroed_page(&mut self) -> *mut () {
        let page = self.alloc_page();
        unsafe {
            // clear the page to 0
            core::ptr::write_bytes(page, 0, PAGE_SIZE);
        }
        page
    }
}