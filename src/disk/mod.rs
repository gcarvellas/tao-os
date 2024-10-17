pub mod ata_pio;
pub mod diskreader;
pub mod diskstreamer;

use alloc::boxed::Box;
use disk::diskstreamer::DiskStreamer;

use crate::fs::find_filesystem;
use crate::{fs::FileSystem, status::ErrorCode};

pub struct Disk {
    id: usize,
    fs: Box<dyn FileSystem>,
}

impl Disk {
    pub fn new(id: usize) -> Result<Self, ErrorCode> {
        let streamer = DiskStreamer::new(id)?;
        let fs = find_filesystem(streamer)?;
        Ok(Self { id, fs })
    }
}
