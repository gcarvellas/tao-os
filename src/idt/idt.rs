// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64

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

#[repr(C)]
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
        let address = interrupt_function as *const (); 
        let bits = address as u64;
        self.offset_1 = (bits & 0xFFFF).try_into().unwrap();
        self.offset_2 = ((bits >> 16) & 0xFFFF).try_into().unwrap();
        self.offset_3 = ((bits >> 32) & 0xFFFFFFFF).try_into().unwrap();
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
        println!("{:?}",  int20h as *const ());
        println!("{}", (int20h as *const ()) as u64);
        println!("{}", _idt_descriptors[0x20].offset_1);
        println!("{}", _idt_descriptors[0x20].offset_2);
        println!("{}", _idt_descriptors[0x20].offset_3);

        unsafe { idt_load(&_idtr_desc) } ;
        return Idt {
            idt_descriptors: _idt_descriptors,
            idtr_desc: _idtr_desc,
        };
    }
}

