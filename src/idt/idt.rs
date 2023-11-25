// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64

use core::{mem::size_of, convert::TryInto};
use crate::{config::TOTAL_INTERRUPTS, io::isr::outb};
use println;
use core::arch::asm;
use alloc::boxed::Box;

extern {
    fn no_interrupt();
    fn int20h();
}

#[no_mangle]
fn int20h_handler() -> () {
    println!("Timing interrupt!");
    outb(0x20, 0x20);
}

#[no_mangle]
fn no_interrupt_handler() -> () {
    outb(0x20, 0x20);
}

#[inline(always)]
pub fn enable_interrupts() -> () {
    unsafe {
        asm! {
            "sti"
        }
    }
}

#[inline(always)]
pub fn disable_interrupts() -> () {
    unsafe {
        asm! {
            "cli"
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtDesc {
    offset_1: u16, // Offset bits 0-15
    selector: u16, // GDT selector
    ist: u8, // bits 0..2 holds Interrupt Stack Table offset, rest of bits zero
    type_attributes: u8, // gate type, dpl, and p fields
    offset_2: u16, // offset bits 16-31
    offset_3: u32, // offset bits 32-63
    zero: u32, // Unused
}

impl IdtDesc {
    fn default() -> IdtDesc {
        return IdtDesc {
            offset_1: 0,
            selector: 0x08, // GDT Code Segment Selector
            ist: 0x00, // Do not use Interrupt Stack Table
            type_attributes: 0x8E, // Interrupt Gate
            offset_2: 0,
            offset_3: 0,
            zero: 0
        }
    }
    fn set(&mut self, interrupt_function: unsafe extern "C" fn() -> ()) -> () {
        // Assumes selector, zero, and type_addr 
        // are set in IdtDesc::default()
        let address = (interrupt_function as *const ()) as u64; 
        self.offset_1 = address as u16;
        self.offset_2 = (address >> 16) as u16;
        self.offset_3 = (address >> 32) as u32;
    }
}

#[repr(C, packed)]
struct IdtrDesc {
    limit: u16, // Size of descriptor table -1
    base: u64 // Base address of IDT
}

impl IdtrDesc {
    fn new(idt_descriptors: *const IdtDesc) -> IdtrDesc {
        return IdtrDesc {
            limit: (size_of::<[IdtDesc; TOTAL_INTERRUPTS]>() - 1) as u16,
            base: idt_descriptors as u64,
        };
    }
}

pub struct Idt {
    idtr_desc: IdtrDesc,
    idt_descriptors: Box<[IdtDesc; TOTAL_INTERRUPTS]>,
}

impl Idt {
    pub fn default() -> Idt {
        let mut _idt_descriptors = Box::new([IdtDesc::default(); TOTAL_INTERRUPTS]);
        let _idtr_desc = IdtrDesc::new(_idt_descriptors.as_ptr());

        for i in 0..TOTAL_INTERRUPTS {
            _idt_descriptors[i].set(no_interrupt);
        }
        _idt_descriptors[0x20].set(int20h);

        // Load IDT Descriptor
        unsafe {
            asm! {
                "lidt [{0}]",
                in(reg) &_idtr_desc
            }
        }
        enable_interrupts();
        loop{}
        return Idt {
            idt_descriptors: _idt_descriptors,
            idtr_desc: _idtr_desc,
        };
    }
}

