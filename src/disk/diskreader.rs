use alloc::boxed::Box;

use crate::status::ErrorCode;

use super::ata_pio::AtaPio;

pub trait DiskReader: Send + Sync {
    fn read(&self, lba: usize, out: &mut [u16], total: usize) -> Result<usize, ErrorCode>;
    fn write(&self, lba: usize, data: &mut [u16]) -> Result<(), ErrorCode>;
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
