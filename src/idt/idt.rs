//https://wiki.osdev.org/Interrupts

use core::{mem::size_of_val, convert::TryInto};
use crate::{config::TOTAL_INTERRUPTS, io::outb};
use println;

extern {
    static mut interrupt_pointer_table: [*mut u8; TOTAL_INTERRUPTS];
    fn idt_load(ptr: *const IdtrDesc);
    fn no_interrupt();
}

#[no_mangle]
pub fn no_interrupt_handler() -> () {
    outb(0x20, 0x20);
}

fn idt_clock() -> () {
    println!("Timing interrupt!");
    outb(0x20, 0x20);
}

#[no_mangle]
pub fn interrupt_handler(interrupt: usize, frame: InterruptFrame) -> () {
    unimplemented!(); // TODO    
    outb(0x20, 0x20);
}

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone)]
struct IdtDesc {
    offset_1: u16, // Offset bits 0-15
    selector: u16, // GDT selector
    zero: u8, // Unused
    type_attr: u8, // Descriptor type and attributes
    offset_2: u16 // Offset bits 16-31
}

impl IdtDesc {
    fn default() -> IdtDesc {
        return IdtDesc {
            offset_1: 0,
            selector: 0x08,
            zero: 0x00,
            type_attr: 0xEE,
            offset_2: 0
        }
    }
    fn set(&mut self, address: *mut u8) -> () {
        // Assumes selector, zero, and type_addr 
        // are set in IdtDesc::default()
        self.offset_1 = ((address as u32) & 0x0000ffff).try_into().unwrap();
        self.offset_2 = ((address as u32) >> 16).try_into().unwrap();
    }
}

#[repr(C)]
#[repr(packed)]
struct IdtrDesc {
    limit: u16, // Size of descriptor table -1
    base: u32 // Base address of IDT
}

impl IdtrDesc {
    fn new(idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS]) -> IdtrDesc {
        return IdtrDesc {
            limit: size_of_val(&idt_descriptors).try_into().unwrap(),
            base: idt_descriptors.as_ptr() as u32
        };
    }
}

#[repr(C)]
#[repr(packed)]
struct InterruptFrame {
    rdi: u64,
    rsi: u64,
    rbp: u64,
    reserved: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    ip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64
}

pub struct Idt {
    idtr_desc: IdtrDesc,
    idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS],
    interrupt_callbacks: [fn() -> (); TOTAL_INTERRUPTS]
}

impl Idt {
    pub fn default() -> Idt {
        let mut _idt_descriptors = [IdtDesc::default(); TOTAL_INTERRUPTS];
        let _idtr_desc = IdtrDesc::new(_idt_descriptors);

        // The standard ISA IRQs start at 20 since we 
        // want to keep the intel CPU interrupts
        let mut _interrupt_callbacks: [fn() -> (); TOTAL_INTERRUPTS] = [no_interrupt_handler; TOTAL_INTERRUPTS];

        for i in 0..TOTAL_INTERRUPTS {
            _idt_descriptors[i].set(unsafe { interrupt_pointer_table[i] });
        }
        _interrupt_callbacks[0x20] = idt_clock;
        unsafe { idt_load(&_idtr_desc) } ;

        return Idt {
            idt_descriptors: _idt_descriptors,
            idtr_desc: _idtr_desc,
            interrupt_callbacks: _interrupt_callbacks
        };
    }
}

