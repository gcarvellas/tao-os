pub mod ata_pio;
pub mod diskreader;
pub mod diskstreamer;

use alloc::{boxed::Box, sync::Arc};
use hashbrown::HashMap;
use spin::{Lazy, RwLock};

use crate::{
    config::SECTOR_SIZE,
    fs::{fs_resolve, FileSystem},
    status::ErrorCode,
};

static DISKS: Lazy<RwLock<HashMap<DiskId, Arc<Disk>>>> = Lazy::new(|| RwLock::new(HashMap::new()));
pub type DiskId = u32;

pub struct Disk {
    pub id: DiskId,
    pub sector_size: u16,
    pub fs: Option<Box<dyn FileSystem>>,
}

impl Disk {
    fn new(id: u32) -> Result<Self, ErrorCode> {
        let mut disk = Self {
            id,
            sector_size: SECTOR_SIZE,
            fs: None,
        };
        match fs_resolve(&mut disk) {
            Ok(fs) => disk.fs = fs,
            Err(err) => return Err(err),
        };
        Ok(disk)
    }

    pub fn get(id: u32) -> Result<Arc<Self>, ErrorCode> {
        {
            let disks = DISKS.read();
            if let Some(disk) = &disks.get(&id) {
                return Ok(Arc::clone(disk));
            }
        }
        let disk = Arc::new(Self::new(id)?);
        {
            let mut disks = DISKS.write();
            disks.insert(id, Arc::clone(&disk));
        }
        Ok(disk)
    }
}
