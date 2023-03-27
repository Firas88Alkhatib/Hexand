#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_trait_impl)]
#![feature(const_slice_index)]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};

#[macro_use]
mod frame_buffer;
mod interrupts;
mod gdt;
// mod acpi;

entry_point!(start);

fn start(boot_info: &'static mut BootInfo) -> ! {
    // let rsdp_addr = boot_info.rsdp_addr.into_option().unwrap();
    let framebuffer_info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();
    frame_buffer::init(framebuffer, framebuffer_info);
    println!("Frame buffer initialized.");
    // acpi::init(rsdp_addr);
    // println!("Advanced Configuration and Power Interface (ACIP) initialized.");
    gdt::init();
    println!("Global Descriptor Table initialized.");
    interrupts::init_idt();
    println!("Interrupts initialized.");
    println!("-----------------------------------------------");
    // frame_buffer::image();
    // x86_64::instructions::interrupts::int3();
      // trigger a page fault
    //   fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // // trigger a stack overflow
    // stack_overflow();


    println!("Checkpoint continue!");
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
   
}