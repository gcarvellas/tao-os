/*
 * 64-bit GDT Implementation
 * References:
 * https://wiki.osdev.org/Global_Descriptor_Table
 */

use crate::{
    config::TOTAL_GDT_SEGMENTS,
    status::ErrorCode,
    task::tss::{Tss, TSS},
};
use bilge::prelude::Number;
use bilge::{
    bitsize,
    prelude::{u2, u24, u4, u40},
    Bitsized, FromBits,
};
use core::convert::TryInto;
use core::{arch::asm, convert::TryFrom};
use spin::Lazy;

// GDT Address bases match the same as the linux kernel
// TODO these values are definitely WRONG! Look at here https://wiki.osdev.org/GDT_Tutorial
// I should resturcutre the GdtStructured so it more follows the content sections of osdev. I don't
// like how the flags are not visible. A gdt structured should have a base, limit, access byte, and
// flags.
// TODO don't forget to load the TSS
// TODO maybe I should restructure my OS to use higher addresses https://wiki.osdev.org/Higher_Half_x86_Bare_Bones
pub static GDT: Lazy<Gdtr> = Lazy::new(|| {
    let tss_address = &*TSS as *const Tss as u64;

    unimplemented!();

    let entries: [GdtStructured; TOTAL_GDT_SEGMENTS] = [
        // NULL Segment
        GdtStructured {
            base: 0x00,
            limit: 0x00,
            r#type: GdtAccess::from(0x00),
        },
        // Kernel Code Segment (x86-64)
        GdtStructured {
            base: 0x00af9b000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0x9A),
        },
        // Kernel Data Segment (x86-64)
        GdtStructured {
            base: 0x00cf93000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0x9A),
        },
        // User Code Segment (x86-64)
        GdtStructured {
            base: 0x00affb000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0xF8),
        },
        // User Data Segment (x86-64)
        GdtStructured {
            base: 0x00cff3000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0xF2),
        },
        // Kernel Code Segment (x86)
        GdtStructured {
            base: 0x00cf9b000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0x9A),
        },
        // Kernel Data Segment (x86)
        GdtStructured {
            base: 0x00cf93000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0x9A),
        },
        // User Code Segment (x86)
        GdtStructured {
            base: 0x00cffb000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0xF8),
        },
        // User Data Segment (x86)
        GdtStructured {
            base: 0x00cff3000000ffff,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0xF2),
        },
        // TSS Segment
        GdtStructured {
            base: tss_address,
            limit: 0xffffffff,
            r#type: GdtAccess::from(0xE9),
        },
    ];
    GdtrStructured { entries }
        .try_into()
        .expect("Failed to load GDTR")
});

#[repr(C, packed)]
#[derive(Default)]
struct Gdt {
    segment: u16,
    base_low: u24,
    access: GdtAccess,
    flags: GdtFlags,
    base_high: u40,
    reserved: u32,
}

#[repr(C, packed)]
struct Gdtr {
    entries: [Gdt; TOTAL_GDT_SEGMENTS],
}

impl Gdtr {
    pub fn load(&self) {
        let addr = self.entries.as_ptr();
        unsafe {
            asm! {
                "lgdt, {0}",
                in(reg) addr
            }
        }
    }
}

#[repr(C, packed)]
#[bitsize(8)]
#[derive(Clone, Copy, FromBits, Default)]
struct GdtAccess {
    accessed: bool,
    rw: bool,
    dc: bool,
    exec: bool,
    not_sys: bool,
    dpl: u2,
    present: bool,
}

#[repr(C, packed)]
#[bitsize(8)]
#[derive(Clone, Copy, FromBits, Default)]
struct GdtFlags {
    limit: u4,
    reserved: bool,
    long_mode: bool,
    sz_32: bool,
    gran_4k: bool,
}

// TODO idk if those sizes are correct
#[derive(Copy, Clone)]
struct GdtStructured {
    base: u64,
    limit: u32,
    r#type: GdtAccess,
}

struct GdtrStructured {
    entries: [GdtStructured; TOTAL_GDT_SEGMENTS],
}

impl TryFrom<GdtrStructured> for Gdtr {
    type Error = ErrorCode;

    fn try_from(source: GdtrStructured) -> Result<Self, Self::Error> {
        let mut entries: [Gdt; TOTAL_GDT_SEGMENTS] = Default::default();

        // TODO maybe that iteration isn't necessary. No bounds check
        source
            .entries
            .iter()
            .enumerate()
            .try_for_each(|(i, entry)| {
                entries[i] = Gdt::try_from(*entry)?;
                Ok::<(), ErrorCode>(())
            })?;

        Ok(Self { entries })
    }
}

// TODO this needs to be rewritten
impl TryFrom<GdtStructured> for Gdt {
    type Error = ErrorCode;

    fn try_from(source: GdtStructured) -> Result<Self, Self::Error> {
        unimplemented!();
        // assert!(!(source.limit > 65536 && (source.limit & 0xFFF) != 0xFFF), "Invalid GDT argument");

        // let mut flags_u8: u8 = match source.limit > 65536 {
        //     true => 0xC0,
        //     false => 0x00,
        // };

        // let source_limit = match source.limit > 65536 {
        //     true => source.limit >> 12,
        //     false => source.limit,
        // };

        // // TODO it is possible that the source limit can be 65536
        // // TODO this needs to be rewritten anyway. I'm pretty sure this stuff is wrong
        // let segment: u16 = source_limit.try_into().unwrap();

        // flags_u8 |= u8::try_from((source_limit >> 16) & 0x0F).unwrap();

        // let flags = GdtFlags::from(flags_u8);

        // let base_low = u24::from(source.base & 0xFFFFFF);
        // let base_high = u40::from(source.base >> 24 & 0xFFFFFFFFFF);

        // let access = source.r#type;

        // Ok(Self {
        //     segment,
        //     base_low,
        //     access,
        //     flags,
        //     base_high,
        //     reserved: 0x0,
        // })
    }
}
