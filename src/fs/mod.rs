use alloc::{boxed::Box, vec::Vec};
use bilge::bitsize;
use bilge::prelude::u7;
use bilge::prelude::Number;
use bilge::Bitsized;
use core::convert::TryFrom;
use fs::fat16::Fat16;

use crate::{disk::diskstreamer::DiskStreamer, status::ErrorCode};

pub mod fat16;
pub mod pparser;

#[repr(C)]
#[bitsize(8)]
pub struct FileStat {
    available: u7,
    read_only: bool,
}

pub enum FileSeekMode {
    SET,
    CUR,
    END,
}

pub trait FileSystem {
    fn fopen<'a>(&self, filename: &'a str, mode_str: &'a str) -> Result<(), ErrorCode>;
    fn fseek(&self, fd: usize, offset: usize, whence: FileSeekMode) -> Result<(), ErrorCode>;
    fn fread(&self, size: u32, nmemb: u32, fd: usize) -> Result<Vec<u16>, ErrorCode>;
    fn fstat(&self, fd: usize, stat: FileStat) -> Result<(), ErrorCode>;
    fn fclose(&self, fd: usize) -> Result<(), ErrorCode>;
    fn fs_resolve(disk: DiskStreamer) -> Result<Self, ErrorCode>
    where
        Self: Sized;
}

pub fn find_filesystem(streamer: DiskStreamer) -> Result<Box<dyn FileSystem>, ErrorCode> {
    match Fat16::fs_resolve(streamer) {
        Ok(val) => return Ok(Box::new(val)),
        Err(ErrorCode::FsNotUs) => (),
        Err(err) => return Err(err),
    };

    Err(ErrorCode::InvArg)
}
