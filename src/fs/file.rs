use alloc::sync::Arc;
use bilge::bitsize;
use bilge::prelude::u7;
use bilge::prelude::Number;
use bilge::Bitsized;
use core::convert::TryFrom;
use spin::Mutex;
use spin::RwLock;

use crate::config::MAX_FILE_DESCRIPTORS;
use crate::disk::Disk;
use crate::status::ErrorCode;

use super::pparser::parse_path;
use super::FileSystem;

static FILE_DESCRIPTORS: RwLock<[Option<Arc<FileDescriptor>>; MAX_FILE_DESCRIPTORS]> =
    RwLock::new([const { None }; MAX_FILE_DESCRIPTORS]);

pub type FileDescriptorIndex = usize;

#[repr(C, packed)]
#[bitsize(8)]
pub struct FileStat {
    available: u7,
    read_only: bool,
}

#[derive(PartialEq)]
pub enum FileMode {
    Read,
    Write,
    Append,
    Invalid,
}

pub enum FileSeekMode {
    Set,
    Cur,
    End,
}

pub struct FileDescriptor {
    index: FileDescriptorIndex,
    disk: Arc<Mutex<Disk>>,
}

impl FileDescriptor {
    pub fn new(disk: Arc<Mutex<Disk>>) -> Result<Arc<Self>, ErrorCode> {
        let mut descriptors = FILE_DESCRIPTORS.write();

        for (i, descriptor) in descriptors.iter_mut().enumerate() {
            if descriptor.is_none() {
                let fd = Arc::new(Self {
                    index: i + 1,
                    disk: Arc::clone(&disk),
                });

                *descriptor = Some(Arc::clone(&fd));

                return Ok(fd);
            }
        }
        Err(ErrorCode::NoFdAvailable)
    }

    pub fn get(fd: FileDescriptorIndex) -> Result<Option<Arc<Self>>, ErrorCode> {
        // Descriptors start at 1
        let index = fd - 1;

        let descriptors = FILE_DESCRIPTORS.read();

        Ok(descriptors
            .get(index)
            .ok_or(ErrorCode::InvArg)?
            .as_ref()
            .map(Arc::clone))
    }
}

fn file_get_mode_by_string(mode_str: &str) -> FileMode {
    match mode_str.as_bytes().first().copied() {
        Some(b'r') => FileMode::Read,
        Some(b'w') => FileMode::Write,
        Some(b'a') => FileMode::Append,
        _ => FileMode::Invalid,
    }
}

pub fn fopen(filename: &str, mode_str: &str) -> Result<FileDescriptorIndex, ErrorCode> {
    let root_path = parse_path(filename)?;

    let drive_no: usize = usize::try_from(root_path.drive_no)?;
    let disk = Disk::get(drive_no)?;

    let mode = file_get_mode_by_string(mode_str);

    if mode == FileMode::Invalid {
        return Err(ErrorCode::InvArg);
    }

    let fd = FileDescriptor::new(disk)?;
    {
        let mut disk = fd.disk.lock();
        match &mut disk.fs {
            None => return Err(ErrorCode::NoFs),
            Some(fs) => fs.fopen(fd.index, root_path.parts, mode)?,
        }
    }
    Ok(fd.index)
}
