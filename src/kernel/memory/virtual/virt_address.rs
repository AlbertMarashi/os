
// each program has a btree of ranges of free virtual memory
// starting from the BEGIN_USERSPACE and ending at the end of virtual memory?.
// the BTreeMap<VirtualAddress, AddressRange>

// https://github.com/repnop/vanadinite/blob/working-services/src/kernel/vanadinite/src/mem/manager/address_map.rs#L53

/// A region of memory allocated to a task
// #[derive(Debug, PartialEq)]
// pub struct AddressRegion {
//     /// The underlying [MemoryRegion], which may or may not be backed by
//     /// physical memory. `None` represents an unoccupied region.
//     pub region: Option<MemoryRegion>,
//     /// The region span
//     pub span: Range<VirtualAddress>
// }

