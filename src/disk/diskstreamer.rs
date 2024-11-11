use super::{
    diskreader::{find_diskreader, DiskReader},
    DiskId,
};
use crate::{config::SECTOR_SIZE, status::ErrorCode};
use alloc::boxed::Box;
use spin::RwLock;

pub struct DiskStreamer {
    reader: Box<dyn DiskReader>,
    pos: RwLock<usize>,
}

impl DiskStreamer {
    pub fn new(disk_id: DiskId) -> Result<Self, ErrorCode> {
        let reader = find_diskreader(disk_id)?;
        Ok(Self {
            reader,
            pos: RwLock::new(0),
        })
    }

    pub fn seek(&self, pos: usize) {
        *self.pos.write() = pos;
    }

    pub fn read(&self, out: &mut [u8], total: usize) -> Result<usize, ErrorCode> {
        if out.len() < total {
            return Err(ErrorCode::InvArg);
        }

        let sector_size: usize = SECTOR_SIZE.into();
        let pos = *self.pos.read();
        let sector = pos / sector_size;
        let offset = pos % sector_size;

        let mut to_read = total.min(sector_size);
        let mut buf = [0; SECTOR_SIZE as usize];

        let mut read_count = self.reader.read(sector, &mut buf, 1)?;

        // Clamp read_count to `to_read` if more was read than needed
        read_count = read_count.min(to_read);

        // If read_count is less, this is due to hardware limitations in the reader
        if read_count < to_read {
            to_read = read_count;
        }

        // Adjust the stream
        {
            *self.pos.write() += to_read;
        }

        for i in 0..read_count {
            let val = buf[offset + i];
            out[i] = val;
        }

        // Check if more data is needed
        if to_read < total {
            to_read += self.read(out, total - to_read)?;
        }

        Ok(to_read)
    }

    pub fn read_into<S: Sized>(&self, buf: &mut [u8]) -> Result<S, ErrorCode> {
        let size = size_of::<S>();

        if self.read(buf, size)? < size {
            return Err(ErrorCode::Io);
        };

        let res: S = unsafe {
            let ptr = buf.as_ptr() as *const S;
            ptr.read()
        };

        Ok(res)
    }

    pub fn write(&self, data: &mut [u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
