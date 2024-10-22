use self::fat::fat16::Fat16;
use self::file::FileSeekMode;
use alloc::boxed::Box;
use fs::file::FileDescriptorIndex;
use fs::file::FileMode;
use fs::file::FileStat;
use fs::pparser::PathPart;

use crate::{disk::Disk, status::ErrorCode};

pub mod fat;
pub mod file;
pub mod pparser;

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;
    fn fopen(
        &mut self,
        fd: FileDescriptorIndex,
        path: PathPart<'_>,
        mode: FileMode,
    ) -> Result<(), ErrorCode>;
    fn fseek(&self, fd: usize, offset: usize, whence: FileSeekMode) -> Result<(), ErrorCode>;
    fn fread(&self, size: u32, nmemb: u32, fd: usize) -> Result<&[u16], ErrorCode>;
    fn fstat(&self, fd: usize, stat: FileStat) -> Result<(), ErrorCode>;
    fn fclose(&self, fd: usize) -> Result<(), ErrorCode>;
    fn fs_resolve(disk: &Disk) -> Result<Self, ErrorCode>
    where
        Self: Sized;
}

pub fn fs_resolve(disk: &mut Disk) -> Result<Option<Box<dyn FileSystem>>, ErrorCode> {
    match Fat16::fs_resolve(disk) {
        Ok(val) => return Ok(Some(Box::new(val))),
        Err(ErrorCode::FsNotUs) => (),
        Err(err) => return Err(err),
    };

    Ok(None)
}
