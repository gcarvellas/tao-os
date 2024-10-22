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
        
        if let Some((i, slot)) = descriptors.iter_mut().enumerate().find(|(_, d)| d.is_none()) {
            let fd = Arc::new(Self {
                index: i + 1,
                disk: Arc::clone(&disk),
            });

            *slot = Some(Arc::clone(&fd));
            return Ok(fd);
        }
        Err(ErrorCode::NoFdAvailable)
    }

    pub fn get(fd: FileDescriptorIndex) -> Result<Arc<Self>, ErrorCode> {
        FILE_DESCRIPTORS
            .read()
            .get(fd - 1) // descriptors start at 1
            .ok_or(ErrorCode::InvArg)?
            .as_ref()
            .map(Arc::clone)
            .ok_or(ErrorCode::InvArg)
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

pub fn fread(out: &mut [u16], size: u32, nmemb: u32, fd: FileDescriptorIndex) -> Result<(), ErrorCode> {
    if size == 0 || nmemb == 0 || fd < 1 {
        return Err(ErrorCode::InvArg);
    }

    let desc = FileDescriptor::get(fd)?;

    {
        let mut disk = desc.disk.lock();
        match &mut disk.fs {
            None => return Err(ErrorCode::NoFs),
            Some(fs) => fs.fread(out, size, nmemb, fd)?
        };
    }
    Ok(())
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
