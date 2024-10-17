/*
 * ATA PIO Mode
 * https://wiki.osdev.org/ATA_PIO_Mode#Polling_the_Status_vs._IRQs
 */

use alloc::vec::Vec;
use bilge::prelude::*;
use core::convert::TryFrom;

use crate::{
    disk::diskreader::DiskReader,
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

pub struct AtaPio28 {
    lba: usize,
}

impl DiskReader for AtaPio28 {
    // TODO Finish
    // references: https://github.com/knusbaum/kernel/blob/master/ata_pio_drv.c
    // reference: https://wiki.osdev.org/ATA_PIO_Mode#IDENTIFY_command

    fn resolve(lba: usize) -> Result<Self, ErrorCode> {
        // TODO this is not correct. Properly check if the drive is supported
        Ok(Self { lba })
    }

    fn write(&self, data: Vec<u16>) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    // TODO support async through IRQ
    // TODO I think this only works for primary disks
    fn read(&self, total: usize) -> Result<Vec<u16>, ErrorCode> {
        // Select master drive and pass part of the LBA
        let lba_h = ((self.lba >> 24) & 0x0F) | 0xE0;
        outb(ATA_PRIMARY_DRIVE_HEAD, lba_h.try_into()?);

        outb(ATA_PRIMARY_ERR, 0x00);

        // Send the total number of sectors we want to read
        outb(ATA_PRIMARY_SECCOUNT, total.try_into()?);

        // Send more of the LBA
        outb(ATA_PRIMARY_LBA_LO, (self.lba & 0xff).try_into()?);
        outb(ATA_PRIMARY_LBA_MID, (self.lba >> 8).try_into()?);
        outb(ATA_PRIMARY_LBA_HI, (self.lba >> 16).try_into()?);

        // Read command
        outb(ATA_PRIMARY_COMM_REGSTAT, 0x20);

        let mut res: Vec<u16> = Vec::with_capacity(total * SECTOR_SIZE);

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
}
