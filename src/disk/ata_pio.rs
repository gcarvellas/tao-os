/*
 * ATA PIO Mode
 * https://wiki.osdev.org/ATA_PIO_Mode#Polling_the_Status_vs._IRQs
 */

use bilge::prelude::*;
use core::convert::{TryFrom, TryInto};

use crate::{
    disk::diskreader::DiskReader,
    io::isr::{insb, insw, outb},
    status::ErrorCode,
};

const ATA_PRIMARY_BASE_ADDRESS: u16 = 0x1F0;
const ATA_SECONDARY_BASE_ADDRESS: u16 = 0x170;

const ATA_28_MASTER_SELECT: usize = 0xE0;
const ATA_28_SLAVE_SELECT: usize = 0xF0;

const ATA_48_MASTER_SELECT: usize = 0x40;
const ATA_48_SLAVE_SELECT: usize = 0x50;

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

const MAX_POLL_ATTEMPTS: usize = 15;

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

// TODO this always assumes compatibility mode and never checks the PCI config
//https://wiki.osdev.org/PCI_IDE_Controller
pub struct AtaPio {
    base_addr: u16,
    select_28: usize,
    select_48: usize,
    lba48_size: Option<u64>,
    lba28_size: u32,
}

enum AtaPioModes {
    Ata48,
    Ata28,
}

fn poll_read(base_addr: u16, out: &mut [u16], total: usize) -> Result<usize, ErrorCode> {
    let mut size = 0;
    for i in 0..total {
        let mut count = 0;
        loop {
            if count >= MAX_POLL_ATTEMPTS {
                return Err(ErrorCode::Io);
            }
            let status = AtaPioStatusRegister::from(insb(base_addr + ATA_COMM_REGSTAT));
            if status.bsy() {
                continue;
            }
            if status.drq() {
                break;
            }
            if status.df() || status.err() {
                return Err(ErrorCode::Io);
            }
            count += 1;
        }
        for j in 0..SECTOR_SIZE {
            *out.get_mut(i * SECTOR_SIZE * j)
                .ok_or(ErrorCode::OutOfBounds)? = insw(base_addr + ATA_DATA);
        }
        size += SECTOR_SIZE;
    }
    Ok(size)
}

impl AtaPio {
    fn new(
        disk_id: usize,
        base_addr: u16,
        is_slave: bool,
        lba28_size: u32,
        lba48_size: Option<u64>,
    ) -> Self {
        Self {
            base_addr,
            select_28: if is_slave {
                ATA_28_SLAVE_SELECT
            } else {
                ATA_28_MASTER_SELECT
            },
            select_48: if is_slave {
                ATA_48_SLAVE_SELECT
            } else {
                ATA_48_MASTER_SELECT
            },
            lba48_size,
            lba28_size,
        }
    }

    fn read28(&self, lba: usize, out: &mut [u16], total: usize) -> Result<usize, ErrorCode> {
        // Select master/slave drive and pass part of the LBA

        let lba_h = ((lba >> 28) & 0x0F) | self.select_28;
        outb(self.base_addr + ATA_DRIVE_HEAD, lba_h.try_into()?);

        // Send the total number of sectors we want to read
        outb(self.base_addr + ATA_SECCOUNT, total.try_into()?);

        // Send more of the LBA
        outb(self.base_addr + ATA_LBA_LO, (lba & 0xff).try_into()?);
        outb(self.base_addr + ATA_LBA_MID, (lba >> 8).try_into()?);
        outb(self.base_addr + ATA_LBA_HI, (lba >> 16).try_into()?);

        // Read command
        outb(self.base_addr + ATA_COMM_REGSTAT, ATA_28_READ);

        poll_read(self.base_addr, out, total)
    }

    fn read48(&self, lba: usize, out: &mut [u16], total: usize) -> Result<usize, ErrorCode> {
        // Select master/slave drive
        outb(self.base_addr + ATA_DRIVE_HEAD, self.select_48.try_into()?);

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

        poll_read(self.base_addr, out, total)
    }

    fn get_mode(&self) -> AtaPioModes {
        match self.lba48_size {
            Some(_) => AtaPioModes::Ata48,
            None => AtaPioModes::Ata28,
        }
    }
}

impl DiskReader for AtaPio {
    // TODO need one mutex for primary and one for secondary
    fn resolve(disk_id: usize) -> Result<Self, ErrorCode> {
        let base_addr = match disk_id {
            0 | 1 => ATA_PRIMARY_BASE_ADDRESS,
            2 | 3 => ATA_SECONDARY_BASE_ADDRESS,
            _ => return Err(ErrorCode::DiskNotUs),
        };

        let is_slave = disk_id == 1 || disk_id == 3;

        let identity = match is_slave {
            true => ATA_SLAVE_IDENTIFY,
            false => ATA_MASTER_IDENTIFY,
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

        let mut data = [0; SECTOR_SIZE];

        let count = poll_read(base_addr, &mut data, 1).map_err(|err| match err {
            ErrorCode::Io => ErrorCode::DiskNotUs,
            other => other,
        })?;

        assert!(count == SECTOR_SIZE);

        // Check LBA48 support using the 10th bit of the 88th value
        let has_lba48 = data[88] & ATA_48_SUPPORTED == 0;

        // the 100-103th values form a u64, which forms the number of lba48 sectors
        let lba48_size: Option<u64> = if has_lba48 {
            let l1 = u64::from(data[100]);
            let l2 = u64::from(data[101]);
            let l3 = u64::from(data[102]);
            let l4 = u64::from(data[103]);

            // Combine the u16 values into a single u64
            Some((l4 << 48) | (l3 << 32) | (l2 << 16) | l1)
        } else {
            None
        };

        // the 60-61st values form a u32, which forms the number of lba28 sectors
        let lba28_size: u32 = {
            let l1 = u32::from(data[60]);
            let l2 = u32::from(data[61]);
            l2 << 16 | l1
        };

        Ok(Self::new(
            disk_id, base_addr, is_slave, lba28_size, lba48_size,
        ))
    }

    fn write(&self, lba: usize, data: &mut [u16]) -> Result<(), ErrorCode> {
        // TODO
        unimplemented!("ata write not implemented yet")
    }

    fn read(&self, lba: usize, out: &mut [u16], total: usize) -> Result<usize, ErrorCode> {
        match self.get_mode() {
            AtaPioModes::Ata48 => Ok(self.read48(lba, out, total)?),
            AtaPioModes::Ata28 => Ok(self.read28(lba, out, total)?),
        }
    }
}
