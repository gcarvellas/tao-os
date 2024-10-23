use alloc::boxed::Box;
use spin::RwLock;

use crate::{config::SECTOR_SIZE, status::ErrorCode};

use super::diskreader::{find_diskreader, DiskReader};

pub struct DiskStreamer {
    reader: Box<dyn DiskReader>,
    pos: RwLock<usize>,
}

impl DiskStreamer {
    pub fn new(disk_id: usize) -> Result<Self, ErrorCode> {
        let reader = find_diskreader(disk_id)?;
        let pos = 0;
        Ok(Self {
            reader,
            pos: RwLock::new(pos),
        })
    }

    pub fn seek(&self, pos: usize) {
        *self.pos.write() = pos;
    }

    pub fn read(&self, out: &mut [u8], total: usize) -> Result<usize, ErrorCode> {
        let pos = *self.pos.read();
        let sector = pos / SECTOR_SIZE;
        let offset = pos % SECTOR_SIZE;
        let mut total_to_read = total.min(SECTOR_SIZE);
        let overflow = (offset + total_to_read) >= SECTOR_SIZE;

        let mut buf = [0; SECTOR_SIZE];

        if overflow {
            total_to_read -= (offset + total_to_read) - SECTOR_SIZE;
        }

        let mut count = self.reader.read(sector, &mut buf, 1)?;
        if total_to_read > count {
            total_to_read = count;
        }

        for i in 0..total_to_read {
            let val = *buf.get(offset + i).ok_or(ErrorCode::OutOfBounds)?;
            *out.get_mut(i).ok_or(ErrorCode::OutOfBounds)? = val;
        }

        // Adjust the stream
        {
            *self.pos.write() += total_to_read;
        }

        if overflow {
            count += self.read(out, total - SECTOR_SIZE)?;
        }

        Ok(count)
    }

    pub fn read_into<T: Sized>(&self, buf: &mut [u8]) -> Result<T, ErrorCode> {
        let size = size_of::<T>();

        if self.read(buf, size)? < size {
            return Err(ErrorCode::Io);
        };

        let res: T = unsafe {
            let ptr = buf.as_ptr() as *const T;
            ptr.read()
        };

        Ok(res)
    }

    pub fn write(&self, data: &mut [u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
