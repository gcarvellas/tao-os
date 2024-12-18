/*
 * ATA PIO Mode
 * https://wiki.osdev.org/ATA_PIO_Mode#Polling_the_Status_vs._IRQs
 */

use alloc::sync::Arc;
use bilge::prelude::*;
use core::convert::TryFrom;
use spin::{Lazy, Mutex};

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::io::isr::{insb, insw, outb};

use crate::{disk::diskreader::DiskReader, status::ErrorCode};

use super::DiskId;

const ATA_PRIMARY_BASE_ADDRESS: u16 = 0x1F0;
const ATA_SECONDARY_BASE_ADDRESS: u16 = 0x170;

const ATA_28_MASTER_SELECT: u8 = 0xE0;
const ATA_28_SLAVE_SELECT: u8 = 0xF0;

const ATA_48_MASTER_SELECT: u8 = 0x40;
const ATA_48_SLAVE_SELECT: u8 = 0x50;

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
const SECTOR_SIZE: usize = 512;

const MAX_POLL_ATTEMPTS: usize = 15;

type DiskLock = Arc<Mutex<()>>;

// ATA PIO actions must be synchronous
static PRIMARY_DRIVE_MUTEX: Lazy<DiskLock> = Lazy::new(|| Arc::new(Mutex::new(())));
static SECONDARY_DRIVE_MUTEX: Lazy<DiskLock> = Lazy::new(|| Arc::new(Mutex::new(())));

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
    id: DiskId,
    base_addr: u16,
    select_28: u8,
    select_48: u8,
    lba48_size: Option<u64>,
    lba28_size: u32,
}

enum AtaPioModes {
    Ata48,
    Ata28,
}

// No lock guarantee makes this unsafe
unsafe fn poll_read(base_addr: u16) -> Result<(), ErrorCode> {
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
            return Ok(());
        }
        if status.df() || status.err() {
            return Err(ErrorCode::Io);
        }
        count += 1;
    }
}

// No lock guarantee makes this unsafe
unsafe fn poll_read_u8(base_addr: u16, out: &mut [u8], total: u16) -> Result<usize, ErrorCode> {
    let mut size = 0;
    for i in 0..usize::from(total) {
        poll_read(base_addr)?;

        // Split the u16 into each of its two values
        for j in (0..SECTOR_SIZE).step_by(2) {
            let offset = i * SECTOR_SIZE + j;

            let data = insw(base_addr + ATA_DATA);
            let [low_byte, high_byte] = data.to_le_bytes();

            out[offset] = low_byte;
            out[offset + 1] = high_byte;
        }
        size += SECTOR_SIZE;
    }
    Ok(size)
}

// No lock guarantee makes this unsafe
unsafe fn poll_read_u16(base_addr: u16, out: &mut [u16], total: u16) -> Result<usize, ErrorCode> {
    let mut size = 0;
    for i in 0..usize::from(total) {
        poll_read(base_addr)?;

        for j in 0..SECTOR_SIZE / 2 {
            let offset = i * SECTOR_SIZE + j;

            let data = insw(base_addr + ATA_DATA);
            out[offset] = data;
        }
        size += SECTOR_SIZE;
    }
    Ok(size)
}

impl AtaPio {
    fn new(
        disk_id: DiskId,
        base_addr: u16,
        is_slave: bool,
        lba28_size: u32,
        lba48_size: Option<u64>,
    ) -> Self {
        Self {
            id: disk_id,
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

    fn read28(&self, lba: usize, out: &mut [u8], total: u8) -> Result<usize, ErrorCode> {
        // Select master/slave drive and pass part of the LBA

        let lba_h: u8 = (lba >> 28).to_le_bytes()[0] | self.select_28;

        let lock = get_lock(self.id)?;

        // unsafe(): safety is handled because of the mutex
        unsafe {
            lock.lock();

            let lba_bytes = lba.to_le_bytes();

            outb(self.base_addr + ATA_DRIVE_HEAD, lba_h);

            // Send the total number of sectors we want to read
            outb(self.base_addr + ATA_SECCOUNT, total);

            // Send more of the LBA
            outb(self.base_addr + ATA_LBA_LO, lba_bytes[0]);
            outb(self.base_addr + ATA_LBA_MID, lba_bytes[1]);
            outb(self.base_addr + ATA_LBA_HI, lba_bytes[2]);

            // Read command
            outb(self.base_addr + ATA_COMM_REGSTAT, ATA_28_READ);

            // This call is safe as long as we have the lock
            poll_read_u8(self.base_addr, out, total.into())
        }
    }

    fn read48(&self, lba: usize, out: &mut [u8], total: u16) -> Result<usize, ErrorCode> {
        let lock = get_lock(self.id)?;

        // unsafe(): safety is handled because of the mutex
        unsafe {
            lock.lock();

            let lba_bytes = lba.to_le_bytes();
            let total_bytes = total.to_le_bytes();

            // Select master/slave drive
            outb(self.base_addr + ATA_DRIVE_HEAD, self.select_48);

            // Sectorcount high
            outb(self.base_addr + ATA_SECCOUNT, total_bytes[1]);

            // lba4, 5, 6
            outb(self.base_addr + ATA_LBA_LO, lba_bytes[3]);
            outb(self.base_addr + ATA_LBA_MID, lba_bytes[4]);
            outb(self.base_addr + ATA_LBA_HI, lba_bytes[5]);

            // Sectorcount low
            outb(self.base_addr + ATA_SECCOUNT, total_bytes[0]);

            // lba1, 2, 3
            outb(self.base_addr + ATA_LBA_LO, lba_bytes[0]);
            outb(self.base_addr + ATA_LBA_MID, lba_bytes[1]);
            outb(self.base_addr + ATA_LBA_HI, lba_bytes[2]);

            // Read command
            outb(self.base_addr + ATA_COMM_REGSTAT, ATA_48_READ);

            // This call is safe as long as we have the lock
            poll_read_u8(self.base_addr, out, total)
        }
    }

    fn get_mode(&self) -> AtaPioModes {
        match self.lba48_size {
            Some(_) => AtaPioModes::Ata48,
            None => AtaPioModes::Ata28,
        }
    }
}

fn is_primary(disk_id: DiskId) -> bool {
    match disk_id {
        0 | 1 => true,
        2 | 3 => false,
        _ => panic!("Invalid disk id {}", disk_id),
    }
}

fn get_lock(disk_id: DiskId) -> Result<DiskLock, ErrorCode> {
    Ok(match is_primary(disk_id) {
        true => Arc::clone(&PRIMARY_DRIVE_MUTEX),
        false => Arc::clone(&SECONDARY_DRIVE_MUTEX),
    })
}

impl DiskReader for AtaPio {
    fn resolve(disk_id: DiskId) -> Result<Self, ErrorCode> {
        let is_primary = is_primary(disk_id);

        let base_addr = match is_primary {
            true => ATA_PRIMARY_BASE_ADDRESS,
            false => ATA_SECONDARY_BASE_ADDRESS,
        };

        let is_slave = disk_id == 1 || disk_id == 3;

        let identity = match is_slave {
            true => ATA_SLAVE_IDENTIFY,
            false => ATA_MASTER_IDENTIFY,
        };

        let lock = get_lock(disk_id)?;

        let mut data = [0; SECTOR_SIZE];
        // unsafe(): safety is handled because of the mutex
        let count = unsafe {
            lock.lock();

            // ATA PIO Identity
            outb(base_addr + ATA_DRIVE_HEAD, identity);

            // Send the total number of sectors we want to read
            outb(base_addr + ATA_SECCOUNT, 0);

            // Send more of the LBA
            outb(base_addr + ATA_LBA_LO, 0);
            outb(base_addr + ATA_LBA_MID, 0);
            outb(base_addr + ATA_LBA_HI, 0);

            outb(base_addr + ATA_COMM_REGSTAT, ATA_IDENTITY);

            poll_read_u16(base_addr, &mut data, 1).map_err(|err| match err {
                ErrorCode::Io => ErrorCode::DiskNotUs,
                other => other,
            })?
        };

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

    fn write(&self, lba: usize, data: &mut [u8]) -> Result<(), ErrorCode> {
        unimplemented!("ata write not implemented yet")
    }

    fn read(&self, lba: usize, out: &mut [u8], total: usize) -> Result<usize, ErrorCode> {
        match self.get_mode() {
            AtaPioModes::Ata48 => match u16::try_from(total) {
                Ok(nmemb) => self.read48(lba, out, nmemb),
                Err(_) => self.read48(lba, out, u16::MAX),
            },
            _ => match u8::try_from(total) {
                Ok(nmemb) => self.read28(lba, out, nmemb),
                Err(_) => self.read28(lba, out, u8::MAX),
            },
        }
    }
}
