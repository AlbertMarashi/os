use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::ptr;

/// Types of devices that can be discovered
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum DeviceType {
    Uart,
    Block,
    Gpu,
    Network,
    Memory,
    Cpu,
    Interrupt,
    Timer,
    Unknown,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Uart => write!(f, "UART"),
            DeviceType::Block => write!(f, "Block Device"),
            DeviceType::Gpu => write!(f, "GPU"),
            DeviceType::Network => write!(f, "Network"),
            DeviceType::Memory => write!(f, "Memory"),
            DeviceType::Cpu => write!(f, "CPU"),
            DeviceType::Interrupt => write!(f, "Interrupt Controller"),
            DeviceType::Timer => write!(f, "Timer"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents a memory region for a device
#[derive(Debug)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
}

/// Represents an interrupt assigned to a device
#[derive(Debug)]
pub struct Interrupt {
    pub irq: usize,
    pub flags: u32,
}

/// Represents a discovered device
pub struct Device {
    pub name: String,
    pub compatible: Vec<String>,
    pub device_type: DeviceType,
    pub memory_regions: Vec<MemoryRegion>,
    pub interrupts: Vec<Interrupt>,
    pub initialized: bool,
}

impl Device {
    pub fn new(name: &str, device_type: DeviceType) -> Self {
        Device {
            name: name.to_string(),
            compatible: Vec::new(),
            device_type,
            memory_regions: Vec::new(),
            interrupts: Vec::new(),
            initialized: false,
        }
    }

    pub fn add_compatible(&mut self, compatible: &str) {
        self.compatible.push(compatible.to_string());
    }

    pub fn add_memory_region(&mut self, base: usize, size: usize) {
        self.memory_regions.push(MemoryRegion { base, size });
    }

    pub fn add_interrupt(&mut self, irq: usize, flags: u32) {
        self.interrupts.push(Interrupt { irq, flags });
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device {{ name: {}, type: {} }}",
            self.name, self.device_type
        )
    }
}

/// Manages all devices in the system
#[derive(Debug)]
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            devices: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Device) {
        info!("Discovered {}: {}", device.device_type, device.name);
        self.devices.push(device);
    }

    pub fn discover_devices(&mut self) {
        discover_devices_from_dtb(self);
    }

    pub fn num_devices(&self) -> usize {
        self.devices.len()
    }
}

// === DEVICE TREE IMPLEMENTATION ===

// Magic value used to identify a DTB (device tree blob)
const FDT_MAGIC: u32 = 0xd00dfeed;

// DTB header structure (simplified)
#[repr(C)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

// DTB token values
const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

// Declare the external dtb_pointer global
extern "C" {
    static dtb_pointer: usize;
}

/// Find the DTB in memory and parse it to discover devices
fn discover_devices_from_dtb(device_manager: &mut DeviceManager) {
    // Get DTB address
    section!("DEVICE TREE", "Locating and validating DTB");
    let dtb_addr = match locate_dtb() {
        Some(addr) => {
            info!("Found DTB at {:#x}", addr);
            addr
        }
        None => {
            error!("Could not locate DTB in memory!");
            panic!("FATAL: Could not locate DTB in memory! System halted.");
        }
    };

    // Read and validate DTB header
    info!("Validating DTB Header");
    let header = unsafe { &*(dtb_addr as *const FdtHeader) };
    if u32::from_be(header.magic) != FDT_MAGIC {
        error!("Invalid DTB magic value!");
        panic!("FATAL: Invalid DTB magic! System halted.");
    }
    info!("Valid DTB found at address {:#x}", dtb_addr);
    end_section!();

    // Get offsets for structure and strings
    section!("DEVICE TREE", "Parsing DTB Structure");

    let struct_offset = u32::from_be(header.off_dt_struct) as usize;
    let strings_offset = u32::from_be(header.off_dt_strings) as usize;
    let struct_ptr = (dtb_addr as usize + struct_offset) as *const u32;
    let strings_ptr = (dtb_addr as usize + strings_offset) as *const u8;

    // Parse DTB structure
    unsafe {
        parse_dtb_structure(struct_ptr, strings_ptr, device_manager);
    }

    end_section!();
}

/// Locate the DTB in memory
fn locate_dtb() -> Option<usize> {
    // Get DTB pointer from QEMU/bootloader
    let dtb_ptr = unsafe { dtb_pointer };

    // Return None if no pointer provided
    if dtb_ptr == 0 {
        return None;
    }

    // Verify it points to a valid DTB by checking magic value
    let header = unsafe { &*(dtb_ptr as *const FdtHeader) };
    let magic = u32::from_be(unsafe { ptr::read_volatile(&header.magic) });

    if magic != FDT_MAGIC {
        return None;
    }

    Some(dtb_ptr)
}

/// Parse the DTB structure to find devices
unsafe fn parse_dtb_structure(
    mut struct_ptr: *const u32,
    strings_ptr: *const u8,
    device_manager: &mut DeviceManager,
) {
    let mut current_path = String::new();

    loop {
        let token = u32::from_be(ptr::read(struct_ptr));
        struct_ptr = struct_ptr.add(1);

        match token {
            FDT_BEGIN_NODE => {
                // Read node name (null-terminated string)
                let node_name_ptr = struct_ptr as *const u8;
                let mut len = 0;
                while ptr::read(node_name_ptr.add(len)) != 0 {
                    len += 1;
                }

                let name_bytes = core::slice::from_raw_parts(node_name_ptr, len);
                let name = core::str::from_utf8_unchecked(name_bytes);

                // Update path
                if !current_path.is_empty() {
                    current_path.push('/');
                }
                current_path.push_str(name);

                // Skip over name (padded to 4-byte alignment)
                let padding = (4 - (len + 1) % 4) % 4;
                struct_ptr = (node_name_ptr.add(len + 1 + padding)) as *const u32;

                // Check if this node is a device
                maybe_create_device(
                    name,
                    &current_path,
                    &mut struct_ptr,
                    strings_ptr,
                    device_manager,
                );
            }

            FDT_END_NODE => {
                // Pop the last component from path
                if let Some(pos) = current_path.rfind('/') {
                    current_path.truncate(pos);
                } else {
                    current_path.clear();
                }
            }

            FDT_PROP => {
                // Skip over property information
                let prop_len = u32::from_be(ptr::read(struct_ptr));
                struct_ptr = struct_ptr.add(2); // Skip length and name offset

                // Skip property value (padded to 4-byte alignment)
                let padded_len = ((prop_len + 3) / 4) * 4;
                struct_ptr = struct_ptr.add((padded_len / 4) as usize);
            }

            FDT_END => break,

            FDT_NOP => {} // No operation

            _ => break, // Stop on unknown token
        }
    }
}

/// Create a device from a DTB node if appropriate
fn maybe_create_device(
    name: &str,
    path: &str,
    _struct_ptr: &mut *const u32,
    _strings_ptr: *const u8,
    device_manager: &mut DeviceManager,
) {
    // Determine device type based on name pattern
    let device_type = if name.starts_with("uart") {
        DeviceType::Uart
    } else if name.starts_with("virtio") {
        DeviceType::Block
    } else if name == "plic" {
        DeviceType::Interrupt
    } else if name == "clint" {
        DeviceType::Timer
    } else if name.starts_with("cpu") {
        DeviceType::Cpu
    } else if name == "memory" {
        DeviceType::Memory
    } else {
        return; // Not a recognized device
    };

    // Create device with minimal required information
    let mut device = Device::new(path, device_type);

    // Add hardcoded memory regions based on device type or parse from DTB properties
    match device_type {
        DeviceType::Uart => {
            device.add_memory_region(0x10000000, 0x100);
            device.add_interrupt(10, 0);
        }
        DeviceType::Block => {
            // Extract address from path if it contains an address
            if let Some(addr_str) = path.rfind('@').map(|pos| &path[pos + 1..]) {
                if let Ok(addr) = usize::from_str_radix(addr_str, 16) {
                    device.add_memory_region(addr, 0x1000);
                    device.add_interrupt(1, 0);
                } else {
                    device.add_memory_region(0x10001000, 0x1000);
                    device.add_interrupt(1, 0);
                }
            } else {
                device.add_memory_region(0x10001000, 0x1000);
                device.add_interrupt(1, 0);
            }
        }
        DeviceType::Interrupt => {
            device.add_memory_region(0x0c000000, 0x4000000);
        }
        DeviceType::Timer => {
            device.add_memory_region(0x02000000, 0x10000);
        }
        _ => {}
    }

    device_manager.add_device(device);
}
