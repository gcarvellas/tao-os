/*
 * 64-bit GDT Implementation
 * References:
 * https://wiki.osdev.org/Task_State_Segment
 */

use spin::Lazy;

pub static TSS: Lazy<Tss> = Lazy::new(Tss::default);

#[repr(C, packed)]
#[derive(Default)]
pub struct Tss {
    reserved_1: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved_2: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved_3: u64,
    iopb: u32,
}
