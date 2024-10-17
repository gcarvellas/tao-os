use crate::{disk::diskreader::DiskReader, status::ErrorCode};

pub struct AtaPio48;

// TODO finish
impl DiskReader for AtaPio48 {
    fn read(&self, total: usize) -> Result<alloc::vec::Vec<u16>, crate::status::ErrorCode> {
        unimplemented!()
    }

    fn write(&self, data: alloc::vec::Vec<u16>) -> Result<(), crate::status::ErrorCode> {
        unimplemented!()
    }

    fn resolve(lba: usize) -> Result<Self, ErrorCode> {
        unimplemented!()
    }
}
