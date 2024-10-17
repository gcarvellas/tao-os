use super::FileSystem;
use crate::disk::diskstreamer::DiskStreamer;
use crate::status::ErrorCode;

pub struct Fat16 {
    disk: DiskStreamer,
}

// TODO implement

impl FileSystem for Fat16 {
    fn fopen<'a>(&self, filename: &'a str, mode_str: &'a str) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn fseek(
        &self,
        fd: usize,
        offset: usize,
        whence: super::FileSeekMode,
    ) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn fread(&self, size: u32, nmemb: u32, fd: usize) -> Result<alloc::vec::Vec<u16>, ErrorCode> {
        unimplemented!()
    }

    fn fstat(&self, fd: usize, stat: super::FileStat) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn fclose(&self, fd: usize) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn fs_resolve(disk: DiskStreamer) -> Result<Self, ErrorCode> {
        unimplemented!()
    }
}
