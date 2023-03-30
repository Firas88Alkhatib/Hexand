#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(const_trait_impl)]
#![feature(const_slice_index)]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::{ entry_point, BootInfo, BootloaderConfig, config::Mapping };
extern crate alloc;

#[macro_use]
mod frame_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod acpi;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(start, config = &BOOTLOADER_CONFIG);

fn start(boot_info: &'static mut BootInfo) -> ! {
    let rsdp_addr = boot_info.rsdp_addr.into_option().unwrap();
    let framebuffer_info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();
    let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let memory_regions = &boot_info.memory_regions;
    // Frame Buffer
    frame_buffer::init(framebuffer, framebuffer_info);
    println!("Frame buffer initialized.");
    unsafe {
        memory::init(physical_memory_offset, memory_regions);
    }
    println!("Memory Management initialized.");
    allocator::init_heap();
    println!("Memory Heap Allocator initialized.");
    let apic_info = acpi::init(rsdp_addr);
    println!("Advanced Configuration and Power Interface (ACIP) initialized.");
    gdt::init();
    println!("Global Descriptor Table initialized.");
    interrupts::init_idt();
    println!("Interrupts initialized.");
    println!("-----------------------------------------------");

    println!("Checkpoint continue!");
    loop {
        interrupts::hlt_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {}