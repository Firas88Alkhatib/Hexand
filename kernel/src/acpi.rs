use core::ptr::NonNull;
use acpi::{ AcpiTables, AcpiHandler, PhysicalMapping };
use x86_64::{ PhysAddr, VirtAddr };



/// it seems this requires allocator , lets get back after implementing memory management

static mut MEMORY_POOL: [u64; 4096] = [0; 4096];

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + (unsafe { MEMORY_POOL[0] }))
}


pub struct MyHandler;

impl Clone for MyHandler {
    fn clone(&self) -> Self {
        MyHandler {}
    }
}

impl AcpiHandler for MyHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virtual_address = phys_to_virt(PhysAddr::new(physical_address as u64));
        PhysicalMapping::new(physical_address, NonNull::new(virtual_address.as_mut_ptr()).unwrap(), size, size, Self)
        
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        println!("ACPI unmap not implemented");
    }
}

// Root System Description Pointer
pub fn init(rsdp_addr: u64) {

    let x:MyHandler = MyHandler;
    let acpi_tables = unsafe { AcpiTables::from_rsdp(x, rsdp_addr as usize).unwrap() };
    let platform_info = acpi_tables.platform_info().unwrap();
    println!("{:?}",platform_info.interrupt_model);
    println!("test");
}