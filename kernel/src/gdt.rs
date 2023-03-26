// Global Descriptn Table
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{ GlobalDescriptorTable, Descriptor, SegmentSelector };

/// Double fault interrupt stack table index
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
struct Gdt {
    gdt: GlobalDescriptorTable,
    selector: Selectors,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        Gdt { gdt, selector: Selectors { code_selector, tss_selector } }
    };
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{ CS, Segment };
    GDT.gdt.load();
    unsafe {
        CS::set_reg(GDT.selector.code_selector);
        load_tss(GDT.selector.tss_selector);
    }
}