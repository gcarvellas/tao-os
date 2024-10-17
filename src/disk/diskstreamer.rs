use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::status::ErrorCode;

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

    // TODO
    pub fn seek(&self, pos: usize) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    pub fn read(&self, total: usize) -> Result<Vec<u16>, ErrorCode> {
        unimplemented!()
    }
}
