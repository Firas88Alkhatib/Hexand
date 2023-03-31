use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;
use lazy_static::lazy_static;

use x86_64::structures::idt::InterruptDescriptorTable;
use acpi::platform::interrupt::Apic;

use x2apic::ioapic::{ IoApic, IrqFlags, RedirectionTableEntry };
use x2apic::lapic::{ LocalApic, LocalApicBuilder, TimerDivide };

use pic8259::ChainedPics;
use crate::gdt;
use crate::memory;
mod handlers;

const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 0x28;
const LAPIC_INTERRUPT_INDEX_OFFSET: u8 = 0x90;

pub static LOCAL_APIC: OnceCell<Spinlock<LocalApic>> = OnceCell::uninit();

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    _IoApic = IOAPIC_INTERRUPT_INDEX_OFFSET, // we reserve this
    Keyboard,
    Mouse = IOAPIC_INTERRUPT_INDEX_OFFSET + 12,
    ApicError = LAPIC_INTERRUPT_INDEX_OFFSET,
    Timer,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IoApicTableIndex {
    Keyboard = 1,
    Mouse = 12,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Exceptions
        idt.breakpoint.set_handler_fn(handlers::breakpoint_handler);
        idt.page_fault.set_handler_fn(handlers::page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(handlers::double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }

        // Interrupts
        idt[InterruptIndex::ApicError as usize].set_handler_fn(handlers::apic_error_handler);
        idt[InterruptIndex::Timer as usize].set_handler_fn(handlers::timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(handlers::keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse as usize].set_handler_fn(handlers::mouse_interrupt_handler);

        idt
    };
}

pub fn init_local_apic(local_apic_address: u64) {
    memory::identity_map(local_apic_address, None);
    let mut local_apic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer as usize)
        .timer_divide(TimerDivide::Div128)
        // .timer_mode(mode)
        .error_vector(InterruptIndex::ApicError as usize)
        .spurious_vector(0xff)
        .set_xapic_base(local_apic_address)
        .build()
        .expect("Failed to build Local APIC");
    unsafe {
        local_apic.enable();
    }
    LOCAL_APIC.init_once(move || Spinlock::new(local_apic));
}

pub unsafe fn init_io_apic(io_apic_address: u64) {
    memory::identity_map(io_apic_address, None);
    let local_apic = LOCAL_APIC.get().expect("Failed to get Local APIC in IO APIC").lock();

    let mut io_apic = IoApic::new(io_apic_address);
    io_apic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);

    register_io_apic_entry(&mut io_apic, local_apic.id() as u8, InterruptIndex::Keyboard as u8, IoApicTableIndex::Keyboard as u8);
    register_io_apic_entry(&mut io_apic, local_apic.id() as u8, InterruptIndex::Mouse as u8, IoApicTableIndex::Mouse as u8);
    drop(local_apic);
}

unsafe fn register_io_apic_entry(ioapic: &mut IoApic, lapic_id: u8, int_index: u8, irq_index: u8) {
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(x2apic::ioapic::IrqMode::Fixed);
    entry.set_dest(lapic_id);
    entry.set_vector(int_index);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
    ioapic.set_table_entry(irq_index, entry);
    ioapic.enable_irq(irq_index);
}

pub fn init_apic(apic_info: Apic) {
    // We need to disable interrupts before initializing the APIC
    x86_64::instructions::interrupts::disable();
    // We need to disable the legacy PIC also as it will conflict with APIC
    disable_legacy_pic();

    IDT.load();

    init_local_apic(apic_info.local_apic_address);
    // We need to enable the interrupts after the APIC initialization
    for io_apic in apic_info.io_apics {
        println!("Initializing IO APIC ID: {}", io_apic.id);
        unsafe {
            init_io_apic(io_apic.address as u64);
        }
    }
    x86_64::instructions::interrupts::enable();
}

pub fn end_of_interrupt() {
    unsafe { LOCAL_APIC.get().expect("Cannot get LAPIC").lock().end_of_interrupt() }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

fn disable_legacy_pic() {
    let pic1_offset: u8 = 0x20; // Master pic
    let pic2_offset: u8 = 0x28; // Slave pic
    unsafe {
        let mut pics = ChainedPics::new(pic1_offset, pic2_offset);
        // Mask all PIC lines
        pics.disable();
    }
}