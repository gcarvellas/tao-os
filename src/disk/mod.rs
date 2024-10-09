/*
 * ATA PIO Mode
 * https://wiki.osdev.org/ATA_PIO_Mode#Polling_the_Status_vs._IRQs
 */

use alloc::vec::Vec;
use bilge::prelude::*;
use core::convert::TryFrom;

use crate::{
    io::isr::{insb, insw, outb},
    status::ErrorCode,
};
use core::convert::TryInto;

const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_ERR: u16 = 0x1F1;
const ATA_PRIMARY_SECCOUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LO: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HI: u16 = 0x1F5;
const ATA_PRIMARY_DRIVE_HEAD: u16 = 0x1F6;
const ATA_PRIMARY_COMM_REGSTAT: u16 = 0x1F7;
const SECTOR_SIZE: usize = 256;

#[bitsize(8)]
#[derive(Clone, Copy, FromBits)]
struct AtaPioStatusRegister {
    err: bool,
    idx: bool,
    corr: bool,
    drq: bool,
    srv: bool,
    df: bool,
    rdy: bool,
    bsy: bool,
}

// TODO support async through IRQ
pub fn ata_pio_read28(lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode> {
    // Select master drive and pass part of the LBA
    let lba_h = ((lba >> 24) & 0x0F) | 0xE0;
    outb(ATA_PRIMARY_DRIVE_HEAD, lba_h.try_into()?);

    outb(ATA_PRIMARY_ERR, 0x00);

    // Send the total number of sectors we want to read
    outb(ATA_PRIMARY_SECCOUNT, total.try_into()?);

    // Send more of the LBA
    outb(ATA_PRIMARY_LBA_LO, (lba & 0xff).try_into()?);
    outb(ATA_PRIMARY_LBA_MID, (lba >> 8).try_into()?);
    outb(ATA_PRIMARY_LBA_HI, (lba >> 16).try_into()?);

    // Read command
    outb(ATA_PRIMARY_COMM_REGSTAT, 0x20);

    let mut res: Vec<u16> = Vec::with_capacity(total*SECTOR_SIZE);

    for _ in 0..total {
        // Wait until buffer is ready
        loop {
            let status = AtaPioStatusRegister::from(insb(ATA_PRIMARY_COMM_REGSTAT));

            // Drive is not ready to transfer data
            if status.bsy() {
                continue;
            }

            // Drive is ready to transfer data
            if status.drq() {
                break;
            }

            // Drive fault error, or other possible error
            // This specific error could be parsed, but regardless an Io error will be thrown
            if status.df() || status.err() {
                return Err(ErrorCode::Io);
            }
        }

        // Copy from hard disk to memory two bytes at a time
        for _ in 0..SECTOR_SIZE {
            res.push(insw(ATA_PRIMARY_DATA));
        }
    }

    Ok(res)
}
