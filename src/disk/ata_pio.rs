/*
 * ATA PIO Mode
 * https://wiki.osdev.org/ATA_PIO_Mode#Polling_the_Status_vs._IRQs
 */

use alloc::vec::Vec;
use bilge::prelude::*;
use core::convert::{TryFrom, TryInto};

use crate::{
    disk::diskreader::DiskReader,
    io::isr::{insb, insw, outb},
    status::ErrorCode,
};

const ATA_PRIMARY_BASE_ADDRESS: u16 = 0x1F0;
const ATA_SECONDARY_BASE_ADDRESS: u16 = 0x170;

const ATA_MASTER_SELECT: usize = 0xE0;
const ATA_SLAVE_SELECT: usize = 0xF0;

const ATA_MASTER_IDENTIFY: u8 = 0xA0;
const ATA_SLAVE_IDENTIFY: u8 = 0xB0;

const ATA_IDENTITY: u8 = 0xEC;
const ATA_28_READ: u8 = 0x20;
const ATA_48_READ: u8 = 0x24;

const ATA_48_SUPPORTED: u16 = 0x200;

const ATA_DATA: u16 = 0;
const ATA_SECCOUNT: u16 = 2;
const ATA_LBA_LO: u16 = 3;
const ATA_LBA_MID: u16 = 4;
const ATA_LBA_HI: u16 = 5;
const ATA_DRIVE_HEAD: u16 = 6;
const ATA_COMM_REGSTAT: u16 = 7;
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

pub struct AtaPio {
    base_addr: u16,
    select: usize,
    lba48_size: Option<u64>,
    lba28_size: u32,
}

enum AtaPioModes {
    Ata48,
    Ata28,
}

fn poll_read(base_addr: u16, total: usize) -> Result<Vec<u16>, ErrorCode> {
    let mut res: Vec<u16> = Vec::with_capacity(total * SECTOR_SIZE);

    for _ in 0..total {
        // Wait until buffer is ready
        loop {
            let status = AtaPioStatusRegister::from(insb(base_addr + ATA_COMM_REGSTAT));

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
            res.push(insw(base_addr + ATA_DATA));
        }
    }

    Ok(res)
}

impl AtaPio {
    fn new(disk_id: usize, lba28_size: u32, lba48_size: Option<u64>) -> Result<Self, ErrorCode> {
        match disk_id {
            0 => Ok(Self {
                base_addr: ATA_PRIMARY_BASE_ADDRESS,
                select: ATA_MASTER_SELECT,
                lba48_size,
                lba28_size,
            }),
            1 => Ok(Self {
                base_addr: ATA_SECONDARY_BASE_ADDRESS,
                select: ATA_SLAVE_SELECT,
                lba48_size,
                lba28_size,
            }),
            _ => Err(ErrorCode::InvArg),
        }
    }

    fn read28(&self, lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode> {
        // Select master/slave drive and pass part of the LBA
        let lba_h = ((lba >> 28) & 0x0F) | self.select;
        outb(self.base_addr + ATA_DRIVE_HEAD, lba_h.try_into()?);

        // Send the total number of sectors we want to read
        outb(self.base_addr + ATA_SECCOUNT, total.try_into()?);

        // Send more of the LBA
        outb(self.base_addr + ATA_LBA_LO, (lba & 0xff).try_into()?);
        outb(self.base_addr + ATA_LBA_MID, (lba >> 8).try_into()?);
        outb(self.base_addr + ATA_LBA_HI, (lba >> 16).try_into()?);

        // Read command
        outb(self.base_addr + ATA_COMM_REGSTAT, ATA_28_READ);

        poll_read(self.base_addr, total)
    }

    fn read48(&self, lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode> {
        // Select master/slave drive and pass part of the LBA
        let lba_h = ((lba >> 28) & 0x0F) | self.select;
        outb(self.base_addr + ATA_DRIVE_HEAD, lba_h.try_into()?);

        // Sectorcount high
        outb(
            self.base_addr + ATA_SECCOUNT,
            ((total >> 8) & 0xFF).try_into()?,
        );

        // lba4, 5, 6
        outb(
            self.base_addr + ATA_LBA_LO,
            ((lba >> 24) & 0xff).try_into()?,
        );
        outb(
            self.base_addr + ATA_LBA_MID,
            ((lba >> 32) & 0xff).try_into()?,
        );
        outb(
            self.base_addr + ATA_LBA_HI,
            ((lba >> 40) & 0xff).try_into()?,
        );

        // Sectorcount low
        outb(self.base_addr + ATA_SECCOUNT, (total & 0xFF).try_into()?);

        // lba1, 2, 3
        outb(self.base_addr + ATA_LBA_LO, (lba & 0xff).try_into()?);
        outb(self.base_addr + ATA_LBA_MID, (lba >> 8).try_into()?);
        outb(self.base_addr + ATA_LBA_HI, (lba >> 16).try_into()?);

        // Read command
        outb(self.base_addr + ATA_COMM_REGSTAT, ATA_48_READ);

        poll_read(self.base_addr, total)
    }

    fn check_bounds(&self, lba: usize, total: usize) -> Result<AtaPioModes, ErrorCode> {
        let read_end = (lba + total) * SECTOR_SIZE;
        if let Some(lba48_size) = self.lba48_size {
            if read_end <= lba48_size.try_into()? {
                return Ok(AtaPioModes::Ata48);
            }
        }

        if read_end <= self.lba28_size.try_into()? {
            return Ok(AtaPioModes::Ata28);
        }

        Err(ErrorCode::InvArg)
    }
}

impl DiskReader for AtaPio {
    fn resolve(disk_id: usize) -> Result<Self, ErrorCode> {
        let base_addr = match disk_id {
            1 => ATA_SECONDARY_BASE_ADDRESS,
            0 => ATA_PRIMARY_BASE_ADDRESS,
            _ => return Err(ErrorCode::DiskNotUs),
        };

        let identity = match disk_id {
            0 => ATA_MASTER_IDENTIFY,
            1 => ATA_SLAVE_IDENTIFY,
            _ => return Err(ErrorCode::DiskNotUs),
        };

        // ATA PIO Identity
        outb(base_addr + ATA_DRIVE_HEAD, identity);

        // Send the total number of sectors we want to read
        outb(base_addr + ATA_SECCOUNT, 0);

        // Send more of the LBA
        outb(base_addr + ATA_LBA_LO, 0);
        outb(base_addr + ATA_LBA_MID, 0);
        outb(base_addr + ATA_LBA_HI, 0);

        outb(base_addr + ATA_COMM_REGSTAT, ATA_IDENTITY);

        let data = poll_read(base_addr, 1).map_err(|err| match err {
            ErrorCode::Io => ErrorCode::DiskNotUs,
            other => other,
        })?;

        // Check LBA48 support using the 10th bit of the 88th value
        let has_lba48 = data.get(88).ok_or(ErrorCode::DiskNotUs)? & ATA_48_SUPPORTED == 0;

        // the 100-103th values form a u64, which forms the number of lba48 sectors
        let lba48_size: Option<u64> = if has_lba48 {
            let l1 = u64::from(*data.get(100).ok_or(ErrorCode::DiskNotUs)?);
            let l2 = u64::from(*data.get(101).ok_or(ErrorCode::DiskNotUs)?);
            let l3 = u64::from(*data.get(102).ok_or(ErrorCode::DiskNotUs)?);
            let l4 = u64::from(*data.get(103).ok_or(ErrorCode::DiskNotUs)?);

            // Combine the u16 values into a single u64
            Some((l4 << 48) | (l3 << 32) | (l2 << 16) | l1)
        } else {
            None
        };

        // the 60-61st values form a u32, which forms the number of lba28 sectors
        let lba28_size: u32 = {
            let l1 = u32::from(*data.get(60).ok_or(ErrorCode::DiskNotUs)?);
            let l2 = u32::from(*data.get(61).ok_or(ErrorCode::DiskNotUs)?);
            l2 << 16 | l1
        };

        Self::new(disk_id, lba28_size, lba48_size)
    }

    fn write(&self, lba: usize, data: Vec<u16>) -> Result<(), ErrorCode> {
        // TODO
        unimplemented!("ata write not implemented yet")
    }

    fn read(&self, lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode> {
        match self.check_bounds(lba, total)? {
            AtaPioModes::Ata48 => Ok(self.read48(lba, total)?),
            AtaPioModes::Ata28 => Ok(self.read28(lba, total)?),
        }
    }
}
