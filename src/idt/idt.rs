//https://wiki.osdev.org/Interrupts

use core::{mem::size_of, convert::TryInto};
use crate::{config::TOTAL_INTERRUPTS, io::outb};
use println;

extern {
    fn idt_load(ptr: *const IdtrDesc);
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

#[repr(C, packed)]
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
    fn set(&mut self, interrupt_function: unsafe extern "C" fn() -> ()) -> () {
        // Assumes selector, zero, and type_addr 
        // are set in IdtDesc::default()
        let address = interrupt_function as *const (); 
        self.offset_1 = ((address as u32) & 0x0000ffff).try_into().unwrap();
        self.offset_2 = ((address as u32) >> 16).try_into().unwrap();
    }
}

#[repr(C, packed)]
struct IdtrDesc {
    limit: u16, // Size of descriptor table -1
    base: u32 // Base address of IDT
}

impl IdtrDesc {
    fn new(idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS]) -> IdtrDesc {
        return IdtrDesc {
            limit:
                {
                    let size: u16 = size_of::<[IdtDesc; TOTAL_INTERRUPTS]>().try_into().unwrap();
                    size - 1
                },
            base: idt_descriptors.as_ptr() as u32
        };
    }
}

pub struct Idt {
    idtr_desc: IdtrDesc,
    idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS],
}

impl Idt {
    pub fn default() -> Idt {
        let mut _idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS] = [IdtDesc::default(); TOTAL_INTERRUPTS];
        let _idtr_desc = IdtrDesc::new(_idt_descriptors);

        for i in 0..TOTAL_INTERRUPTS {
            _idt_descriptors[i].set(no_interrupt);
        }
        _idt_descriptors[0x20].set(int20h);

        unsafe { idt_load(&_idtr_desc) } ;
        return Idt {
            idt_descriptors: _idt_descriptors,
            idtr_desc: _idtr_desc,
        };
    }
}

