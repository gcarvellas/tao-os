use alloc::vec::Vec;

use crate::{
    io::isr::{insb, insw, outb},
    status::ErrorCode,
};
use core::convert::TryInto;

// TODO support async https://os.phil-opp.com/async-await/
pub fn disk_read_sector(lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode> {
    // Select master drive and pass part of the LBA
    let lba_h = (lba >> 24) | 0xE0;
    outb(0x1F6, lba_h.try_into()?);

    // Send the total number of sectors we want to read
    outb(0x1F2, total.try_into()?);

    // Send more of the LBA
    outb(0x1F3, (lba & 0xff).try_into()?);
    outb(0x1F4, (lba >> 8).try_into()?);
    outb(0x1F5, (lba >> 16).try_into()?);

    // Read command
    outb(0x1F7, 0x20);

    let mut res: Vec<u16> = Vec::with_capacity(total);

    for _ in 0..total {
        // Wait until buffer is ready
        let mut c: u8 = insb(0x1F7);
        while c & 0x08 == 0x00 {
            c = insb(0x1F7);
        }

        // Copy from hard disk to memory two bytes at a time
        for _ in 0..256 {
            res.push(insw(0x1F0));
        }
    }

    Ok(res)
}
