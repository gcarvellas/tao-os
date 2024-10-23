use alloc::sync::Arc;
use bilge::bitsize;
use bilge::prelude::u7;
use bilge::prelude::Number;
use bilge::Bitsized;
use bilge::DebugBits;
use core::convert::TryFrom;
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
#[derive(Default, DebugBits)]
pub struct FileStatFlags {
    available: u7,
    pub read_only: bool,
}

#[derive(Debug)]
pub struct FileStat {
    pub flags: FileStatFlags,
    pub filesize: u32,
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
    disk: Arc<Disk>,
}

impl FileDescriptor {
    pub fn new(disk: Arc<Disk>) -> Result<Arc<Self>, ErrorCode> {
        let mut descriptors = FILE_DESCRIPTORS.write();

        if let Some((i, slot)) = descriptors
            .iter_mut()
            .enumerate()
            .find(|(_, d)| d.is_none())
        {
            let fd = Arc::new(Self {
                index: i + 1,
                disk: Arc::clone(&disk),
            });

            *slot = Some(Arc::clone(&fd));
            return Ok(fd);
        }
        Err(ErrorCode::NoFdAvailable)
    }

    pub fn get(fd: FileDescriptorIndex) -> Result<Option<Arc<Self>>, ErrorCode> {
        Ok(FILE_DESCRIPTORS
            .read()
            .get(fd - 1) // descriptors start at 1
            .ok_or(ErrorCode::InvArg)?
            .as_ref()
            .map(Arc::clone))
    }

    fn remove(fd: FileDescriptorIndex) {
        let mut fds = FILE_DESCRIPTORS.write();
        let desc = match fds.get_mut(fd - 1) {
            None => return,
            Some(desc) => desc,
        };
        *desc = None;
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

pub fn fread(
    out: &mut [u8],
    size: u32,
    nmemb: u32,
    fd: FileDescriptorIndex,
) -> Result<(), ErrorCode> {
    if size == 0 || nmemb == 0 || fd < 1 {
        return Err(ErrorCode::InvArg);
    }

    let desc = FileDescriptor::get(fd)?.ok_or(ErrorCode::InvArg)?;

    {
        match &desc.disk.fs {
            None => return Err(ErrorCode::NoFs),
            Some(fs) => fs.fread(out, size, nmemb, fd)?,
        };
    }
    Ok(())
}

pub fn fstat(fd: FileDescriptorIndex) -> Result<FileStat, ErrorCode> {
    if fd < 1 {
        return Err(ErrorCode::InvArg);
    }

    let desc = FileDescriptor::get(fd)?.ok_or(ErrorCode::InvArg)?;

    Ok(match &desc.disk.fs {
        None => return Err(ErrorCode::NoFs),
        Some(fs) => fs.fstat(fd)?,
    })
}

pub fn fseek(
    fd: FileDescriptorIndex,
    offset: usize,
    whence: FileSeekMode,
) -> Result<(), ErrorCode> {
    if fd < 1 {
        return Err(ErrorCode::InvArg);
    }

    let desc = FileDescriptor::get(fd)?.ok_or(ErrorCode::InvArg)?;

    match &desc.disk.fs {
        None => return Err(ErrorCode::NoFs),
        Some(fs) => fs.fseek(fd, offset, whence)?,
    };

    Ok(())
}

pub fn fclose(fd: FileDescriptorIndex) -> Result<(), ErrorCode> {
    let desc = match FileDescriptor::get(fd)? {
        None => return Ok(()),
        Some(desc) => desc,
    };

    match &desc.disk.fs {
        None => return Err(ErrorCode::NoFs),
        Some(fs) => fs.fclose(fd),
    }

    FileDescriptor::remove(fd);

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
        match &fd.disk.fs {
            None => return Err(ErrorCode::NoFs),
            Some(fs) => fs.fopen(fd.index, root_path.parts, mode)?,
        }
    }
    Ok(fd.index)
}
