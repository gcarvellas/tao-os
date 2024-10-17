use alloc::{boxed::Box, vec::Vec};

use crate::status::ErrorCode;

use super::ata_pio::{ata_pio_28::AtaPio28, ata_pio_48::AtaPio48};

pub trait DiskReader {
    fn read(&self, total: usize) -> Result<Vec<u16>, ErrorCode>;
    fn write(&self, data: Vec<u16>) -> Result<(), ErrorCode>;
    fn resolve(index: usize) -> Result<Self, ErrorCode>
    where
        Self: Sized;
}

pub fn find_diskreader(index: usize) -> Result<Box<dyn DiskReader>, ErrorCode> {
    match AtaPio48::resolve(index) {
        Ok(val) => return Ok(Box::new(val)),
        Err(ErrorCode::DiskNotUs) => (),
        Err(err) => return Err(err),
    };

    match AtaPio28::resolve(index) {
        Ok(val) => return Ok(Box::new(val)),
        Err(ErrorCode::DiskNotUs) => (),
        Err(err) => return Err(err),
    };

    Err(ErrorCode::InvArg)
}
