#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};

entry_point!(start);

fn start(boot_info: &'static mut BootInfo) -> ! {

    let _info = boot_info.framebuffer.as_ref().unwrap().info();
    let _framebuffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
   
}