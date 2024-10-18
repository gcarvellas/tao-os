use alloc::{boxed::Box, vec::Vec};

use crate::status::ErrorCode;

use super::ata_pio::AtaPio;

pub trait DiskReader {
    fn read(&self, lba: usize, total: usize) -> Result<Vec<u16>, ErrorCode>;
    fn write(&self, lba: usize, data: Vec<u16>) -> Result<(), ErrorCode>;
    fn resolve(index: usize) -> Result<Self, ErrorCode>
    where
        Self: Sized;
}

pub fn find_diskreader(disk_id: usize) -> Result<Box<dyn DiskReader>, ErrorCode> {
    match AtaPio::resolve(disk_id) {
        Ok(val) => return Ok(Box::new(val)),
        Err(ErrorCode::DiskNotUs) => (),
        Err(err) => return Err(err),
    };

    Err(ErrorCode::InvArg)
}
