/*
 * 64-bit IDT implementation
 * References:
 * https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
 */

use spin::Lazy;
use static_assertions::const_assert_eq;

use crate::{config::TOTAL_INTERRUPTS, io::isr::outb, status::ErrorCode};
use core::arch::asm;
use core::mem::size_of;

pub static IDT: Lazy<Idt> = Lazy::new(|| Idt::new().expect("Failed to initialize IDT"));

extern "C" {
    fn no_interrupt();
    fn int20h();
}

#[no_mangle]
fn int20h_handler() {
    // Safety: expected behavior
    unsafe { master_pic_ack() };
}

#[no_mangle]
fn no_interrupt_handler() {
    // Safety: expected behavior
    unsafe { master_pic_ack() };
}

#[inline(always)]
unsafe fn master_pic_ack() {
    outb(0x20, 0x20);
}

/// SAFETY:
///
/// If initializers aren't setup properly, interrupts will cause unexpected behavior
pub unsafe fn enable_interrupts() {
    asm! {
        "sti"
    }
}

/// SAFETY:
///
/// Ensure that this is properly re-enabled or else most hardware won't work
pub unsafe fn disable_interrupts() {
    asm! {
        "cli"
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtDesc {
    offset_1: u16,       // Offset bits 0-15
    selector: u16,       // GDT selector
    ist: u8,             // bits 0..2 holds Interrupt Stack Table offset, rest of bits zero
    type_attributes: u8, // gate type, dpl, and p fields
    offset_2: u16,       // offset bits 16-31
    offset_3: u32,       // offset bits 32-63
    zero: u32,           // Unused
}

impl IdtDesc {
    fn default() -> Self {
        Self {
            offset_1: 0,
            selector: 0x08,        // GDT Code Segment Selector
            ist: 0x00,             // Do not use Interrupt Stack Table
            type_attributes: 0x8E, // Interrupt Gate
            offset_2: 0,
            offset_3: 0,
            zero: 0,
        }
    }
    fn set(&mut self, interrupt_function: unsafe extern "C" fn() -> ()) -> Result<(), ErrorCode> {
        // Assumes selector, zero, and type_addr
        // are set in IdtDesc::default()
        let address = (interrupt_function as *const ()) as u64;

        let adr_bytes = address.to_le_bytes();
        self.offset_1 = u16::from_le_bytes([adr_bytes[0], adr_bytes[1]]);
        self.offset_2 = u16::from_le_bytes([adr_bytes[2], adr_bytes[3]]);
        self.offset_3 =
            u32::from_le_bytes([adr_bytes[4], adr_bytes[5], adr_bytes[6], adr_bytes[7]]);
        Ok(())
    }
}

#[repr(C, packed)]
struct IdtrDesc {
    limit: u16, // Size of descriptor table -1
    base: u64,  // Base address of IDT
}

impl IdtrDesc {
    fn new(idt_descriptors: *const IdtDesc) -> Result<Self, ErrorCode> {
        const_assert_eq!(size_of::<[IdtDesc; TOTAL_INTERRUPTS]>() - 1, 4095);

        Ok(Self {
            limit: 4095,
            base: idt_descriptors as u64,
        })
    }
}

pub struct Idt {
    idtr_desc: IdtrDesc,
    _idt_descriptors: [IdtDesc; TOTAL_INTERRUPTS],
}

impl Idt {
    pub fn load(&self) {
        unsafe {
            asm! {
                "lidt [{0}]",
                in(reg) &self.idtr_desc
            }
        }
    }
    pub fn new() -> Result<Self, ErrorCode> {
        let mut idt_descriptors = [IdtDesc::default(); TOTAL_INTERRUPTS];
        let idtr_desc = IdtrDesc::new(idt_descriptors.as_ptr())?;

        for descriptor in idt_descriptors.iter_mut() {
            descriptor.set(no_interrupt)?;
        }

        idt_descriptors[0x20].set(int20h)?;

        Ok(Self {
            _idt_descriptors: idt_descriptors,
            idtr_desc,
        })
    }
}
