use x86_64::structures::idt::{ InterruptStackFrame, PageFaultErrorCode };
use x86_64::registers::control::Cr2;
use x86_64::instructions::port::Port;

use crate::interrupts::hlt_loop;
use super::end_of_interrupt;
use crate::task::keyboard;

/**
 * ------------------------------Exception Handlers------------------------------
 */
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {error_code:?}");
    println!("{stack_frame:#?}");
    hlt_loop();
}

/**
 * ------------------------------Interrupt Handlers------------------------------
 */

pub extern "x86-interrupt" fn apic_error_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let lapic = super::LOCAL_APIC.get().expect("Failed to get Local APIC in apic error handler").lock();
        let flags = lapic.error_flags();
        panic!("EXCEPTION: APIC ERROR: {:#?}", flags);
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    end_of_interrupt();
}
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    keyboard::add_scancode(scancode);
    end_of_interrupt();
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let packet: u8 = unsafe { port.read() };
    print!("Mouse PKT: {packet}");
    end_of_interrupt();
}