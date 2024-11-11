use self::fat::fat16::Fat16;
use self::file::FileSeekMode;
use crate::fs::file::FileDescriptorIndex;
use crate::fs::file::FileMode;
use crate::fs::file::FileStat;
use crate::fs::pparser::PathPart;
use alloc::boxed::Box;

use crate::{disk::Disk, status::ErrorCode};

pub mod fat;
pub mod file;
pub mod pparser;

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;
    fn fopen(
        &self,
        fd: FileDescriptorIndex,
        path: PathPart<'_>,
        mode: FileMode,
    ) -> Result<(), ErrorCode>;
    fn fseek(
        &self,
        fd: FileDescriptorIndex,
        offset: usize,
        whence: FileSeekMode,
    ) -> Result<(), ErrorCode>;
    fn fread(
        &self,
        out: &mut [u8],
        size: usize,
        nmemb: usize,
        fd: FileDescriptorIndex,
    ) -> Result<usize, ErrorCode>;
    fn fstat(&self, fd: FileDescriptorIndex) -> Result<FileStat, ErrorCode>;
    fn fclose(&self, fd: FileDescriptorIndex);
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
