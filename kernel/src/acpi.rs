use core::ptr::NonNull;
use acpi::{ AcpiTables, AcpiHandler, PhysicalMapping, platform::interrupt::Apic };
use x86_64::VirtAddr;

#[derive(Clone)]
pub struct ACPIHandler {
    physical_memory_offset: u64,
}

impl AcpiHandler for ACPIHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virtual_address = VirtAddr::new((physical_address as u64) + self.physical_memory_offset);
        PhysicalMapping::new(physical_address, NonNull::new(virtual_address.as_mut_ptr()).unwrap(), size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // println!("ACPI unmap not implemented");
    }
}

// Root System Description Pointer
pub fn init(rsdp_addr: u64, physical_memory_offset: u64) -> Apic {
    let acpi_handler = ACPIHandler { physical_memory_offset };
    let acpi_tables = unsafe { AcpiTables::from_rsdp(acpi_handler, rsdp_addr as usize).unwrap() };
    let platform_info = acpi_tables.platform_info().unwrap();
    let processor_info = platform_info.processor_info.expect("Failed to get processor info");
    println!("---------------ACPI---------------");
    println!("Power Profile: {:?}", platform_info.power_profile);
    println!("Boot Processor: {:?}", processor_info.boot_processor);
    println!("Application Processors: {:?}", processor_info.application_processors);
    println!("---------------ACPI---------------");
    match platform_info.interrupt_model {
        acpi::InterruptModel::Apic(acpi_info) => {
            return acpi_info;
        }
        _ => {
            panic!("Failed to get interrupt model from ACPI");
        }
    }
}