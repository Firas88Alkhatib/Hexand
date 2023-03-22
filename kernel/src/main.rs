#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_trait_impl)]
#![feature(const_slice_index)]

use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};

mod frame_buffer;

entry_point!(start);

fn start(boot_info: &'static mut BootInfo) -> ! {
    let info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();
    frame_buffer::init(framebuffer, info);

    println!("Hello World");
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
   
}