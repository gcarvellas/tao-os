pub mod ata_pio;
pub mod diskreader;
pub mod diskstreamer;

use alloc::{boxed::Box, sync::Arc};
use hashbrown::HashMap;
use spin::{Lazy, Mutex, RwLock};

use crate::{
    config::SECTOR_SIZE,
    fs::{fs_resolve, FileSystem},
    status::ErrorCode,
};

static DISKS: Lazy<RwLock<HashMap<usize, Arc<Mutex<Disk>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Disk {
    pub id: usize,
    pub sector_size: usize,
    pub fs: Option<Box<dyn FileSystem>>,
}

impl Disk {
    fn new(id: usize) -> Result<Self, ErrorCode> {
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

    pub fn get(id: usize) -> Result<Arc<Mutex<Self>>, ErrorCode> {
        {
            let disks = DISKS.read();
            if let Some(disk) = &disks.get(&id) {
                return Ok(Arc::clone(disk));
            }
        }
        let disk = Arc::new(Mutex::new(Self::new(id)?));
        {
            let mut disks = DISKS.write();
            disks.insert(id, Arc::clone(&disk));
        }
        Ok(disk)
    }
}
