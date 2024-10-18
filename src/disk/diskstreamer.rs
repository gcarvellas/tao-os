use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{config::SECTOR_SIZE, status::ErrorCode};

use super::diskreader::{find_diskreader, DiskReader};

pub struct DiskStreamer {
    reader: Box<dyn DiskReader>,
    pos: usize,
}

impl DiskStreamer {
    pub fn new(disk_id: usize) -> Result<Self, ErrorCode> {
        let reader = find_diskreader(disk_id)?;
        let pos = 0;
        Ok(Self { reader, pos })
    }

    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn read(&mut self, total: usize) -> Result<Vec<u16>, ErrorCode> {
        let sector = self.pos / SECTOR_SIZE;
        let offset = self.pos % SECTOR_SIZE;
        let mut total_to_read = if total > SECTOR_SIZE {
            SECTOR_SIZE
        } else {
            total
        };

        let overflow = (offset + total_to_read) >= SECTOR_SIZE;

        if overflow {
            total_to_read -= (offset + total_to_read) - SECTOR_SIZE;
        }

        let mut buf = self.reader.read(sector, 1)?;
        self.pos += total_to_read;

        if overflow {
            buf.extend(self.read(total - SECTOR_SIZE)?);
        }

        Ok(buf)
    }

    pub fn write(&mut self, data: Vec<u16>) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
