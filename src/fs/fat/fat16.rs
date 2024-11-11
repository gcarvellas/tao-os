use crate::fs::{FileSeekMode, FileStat};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bilge::bitsize;
use bilge::prelude::Number;
use bilge::Bitsized;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::mem::size_of;
use hashbrown::HashMap;
use spin::RwLock;
use static_assertions::const_assert_eq;

use crate::config::MAX_PATH;
use crate::disk::diskstreamer::DiskStreamer;
use crate::disk::Disk;
use crate::fs::file::FileDescriptorIndex;
use crate::fs::file::FileStatFlags;
use crate::fs::pparser::PathPart;
use crate::fs::FileMode;
use crate::fs::FileSystem;
use crate::status::ErrorCode;

// Fat16 spec constants/structs

const SIGNATURE: u8 = 0x29;
const FAT_ENTRY_SIZE: u16 = 0x02;

const BAD_SECTOR: u16 = 0xFF7;
const BOOT_SECTOR: u16 = 0xFF0;
const EXTENDED_BOOT_SECTOR: u16 = 0xFF6;
const BLANK_SECTOR: u16 = 0x00;
const EOF: u16 = 0xFFF;

const RESERVED: u16 = 0xFF8;
const BLANK_RECORD: u8 = 0x00;
const UNUSED: u8 = 0xE5;

#[bitsize(8)]
#[derive(Clone, Copy)]
struct FatFileAttributes {
    read_only: bool,
    hidden: bool,
    system: bool,
    volume_label: bool,
    subdirectory: bool,
    archived: bool,
    device: bool,
    reserved: bool,
}

#[repr(C, packed)]
struct FatHeaderExtended {
    drive_number: u8,
    win_nt_bit: u8,
    signature: u8,
    volume_id: u32,
    volume_id_string: [u8; 11],
    system_id_string: [u8; 8],
}

#[repr(C, packed)]
struct FatHeader {
    short_jmp_ins: [u8; 3],
    oem_identifier: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_copies: u8,
    root_dir_entries: u16,
    number_of_sectors: u16,
    media_type: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    sectors_big: u32,
}

#[repr(C, packed)]
struct FatH {
    primary_header: FatHeader,
    extended_header: FatHeaderExtended,
}

#[repr(C, packed)]
#[derive(Clone)]
struct FatDirectoryItem {
    filename: [u8; 8],
    ext: [u8; 3],
    attribute: FatFileAttributes,
    reserved: u8,
    creation_time_tenths_of_a_sec: u8,
    creation_time: u16,
    creation_date: u16,
    last_access: u16,
    high_16_bits_first_cluster: u16,
    last_mod_time: u16,
    last_mod_date: u16,
    low_16_bits_first_cluster: u16,
    filesize: u32,
}

impl FatDirectoryItem {
    /*
     * Used to get the size as a u16 without type converting
     */
    const fn size() -> u16 {
        const_assert_eq!(size_of::<FatDirectoryItem>(), 32);
        32
    }

    unsafe fn from_u8_slice(slice: Vec<u8>, size: usize) -> Vec<Self> {
        let ptr = slice.as_ptr() as *const Self;
        let fat_items = core::slice::from_raw_parts(ptr, size);
        fat_items.to_vec()
    }

    fn first_cluster(&self) -> u16 {
        self.high_16_bits_first_cluster | self.low_16_bits_first_cluster
    }
}

// Internal Structures

struct FatDirectory {
    items: Vec<FatDirectoryItem>,
    sector_pos: u16,
    ending_sector_pos: u16,
}

impl FatDirectory {
    fn get_root(
        disk: &Disk,
        private_header: &FatH,
        directory_stream: &DiskStreamer,
    ) -> Result<Self, ErrorCode> {
        let primary_header = &private_header.primary_header;
        let root_dir_sector_pos: u16 = u16::from(primary_header.fat_copies)
            * primary_header.sectors_per_fat
            + primary_header.reserved_sectors;
        let root_dir_entries = primary_header.root_dir_entries;

        let root_dir_size: u16 = root_dir_entries * FatDirectoryItem::size();

        let total_items =
            get_total_items_for_directory(disk.sector_size, root_dir_sector_pos, directory_stream)?;

        let pos = sector_to_absolute(disk.sector_size, root_dir_sector_pos);
        directory_stream.seek(pos);

        let mut items = Vec::with_capacity(total_items.into());

        for _ in 0..total_items {
            let mut dir_buf = [0; size_of::<FatDirectoryItem>()];
            let dir: FatDirectoryItem = directory_stream.read_into(&mut dir_buf)?;
            directory_stream.read_into(&mut dir_buf)?;

            items.push(dir);
        }

        let ending_sector_pos: u16 = root_dir_sector_pos + (root_dir_size / disk.sector_size);

        Ok(Self {
            items,
            sector_pos: root_dir_sector_pos,
            ending_sector_pos,
        })
    }
}

enum FatItem {
    File(FatDirectoryItem),
    Directory(FatDirectory),
}

struct FatFileDescriptor {
    item: Arc<FatItem>,
    pos: usize,
}

struct FatPrivate {
    header: FatH,
    root_directory: Arc<FatDirectory>,
    cluster_read_stream: DiskStreamer,
    fat_read_stream: DiskStreamer,
    directory_stream: DiskStreamer,
}

impl FatPrivate {
    fn new(
        disk: &Disk,
        header: FatH,
        root_directory: FatDirectory,
        directory_stream: DiskStreamer,
    ) -> Result<Self, ErrorCode> {
        Ok(Self {
            header,
            root_directory: Arc::new(root_directory),
            cluster_read_stream: DiskStreamer::new(disk.id)?,
            fat_read_stream: DiskStreamer::new(disk.id)?,
            directory_stream,
        })
    }
}

pub struct Fat16 {
    private: FatPrivate,
    fds: RwLock<HashMap<FileDescriptorIndex, FatFileDescriptor>>,
    sector_size: u16,
}

impl Fat16 {
    fn get_directory_entry(
        &self,
        root_directory: Arc<FatDirectory>,
        path: PathPart,
    ) -> Result<FatItem, ErrorCode> {
        let mut path_mut = path.clone();

        let root_name = path_mut.next().ok_or(ErrorCode::InvArg)?;

        let root_item = self.find_item_in_directory(&root_directory, root_name)?;

        let mut current_item = root_item;
        for name in path_mut {
            match current_item {
                FatItem::Directory(_) => return Err(ErrorCode::BadPath),
                FatItem::File(file) => {
                    let item_as_directory = self.load_fat_directory(&file)?;
                    current_item = self.find_item_in_directory(&item_as_directory, name)?
                }
            }
        }
        Ok(current_item)
    }

    fn find_item_in_directory(
        &self,
        directory: &FatDirectory,
        name: &str,
    ) -> Result<FatItem, ErrorCode> {
        for item in &directory.items {
            let tmp_filename = get_full_relative_filename(item)?;

            if tmp_filename == name {
                return self.new_fat_item_for_directory_item(item.clone()); // TODO no clone!
            }
        }
        Err(ErrorCode::NotFound)
    }

    fn new_fat_item_for_directory_item(
        &self,
        item: FatDirectoryItem,
    ) -> Result<FatItem, ErrorCode> {
        Ok(match item.attribute.subdirectory() {
            true => {
                let directory = self.load_fat_directory(&item)?;
                FatItem::Directory(directory)
            }
            false => FatItem::File(item),
        })
    }

    fn get_first_fat_sector(&self) -> u16 {
        self.private.header.primary_header.reserved_sectors
    }

    fn get_fat_entry(&self, cluster: u16) -> Result<u16, ErrorCode> {
        let fat_table_position = self.get_first_fat_sector() * self.sector_size;
        self.private
            .fat_read_stream
            .seek(usize::from(fat_table_position * cluster * FAT_ENTRY_SIZE));

        let mut out: [u8; 2] = [0; 2];
        let size = size_of::<[u8; 2]>();
        self.private.fat_read_stream.read(&mut out, size)?;

        Ok(u16::from_le_bytes(out))
    }

    fn get_cluster_for_offset(
        &self,
        starting_cluster: u16,
        offset: usize,
        size_of_cluster_bytes: usize,
    ) -> Result<u16, ErrorCode> {
        let mut cluster_to_use = starting_cluster;
        let clusters_ahead = offset / size_of_cluster_bytes;
        for _ in 0..clusters_ahead {
            let entry: u16 = self.get_fat_entry(cluster_to_use)?;

            cluster_to_use = match entry {
                BLANK_SECTOR | RESERVED | EOF | BAD_SECTOR | BOOT_SECTOR | EXTENDED_BOOT_SECTOR => {
                    return Err(ErrorCode::Io)
                }
                _ => entry,
            };
        }
        Ok(cluster_to_use)
    }

    fn read_internal(
        &self,
        cluster: u16,
        offset: usize,
        mut total: usize,
        out: &mut [u8],
    ) -> Result<(), ErrorCode> {
        let sector_size: usize = self.sector_size.into();
        let size_of_cluster_bytes: usize =
            usize::from(self.private.header.primary_header.sectors_per_cluster) * sector_size;
        let cluster_to_use = self.get_cluster_for_offset(cluster, offset, size_of_cluster_bytes)?;

        let offset_from_cluster = offset % size_of_cluster_bytes;

        let starting_sector: usize = self.cluster_to_sector(cluster_to_use).into();
        let starting_pos: usize = starting_sector * sector_size + offset_from_cluster;

        while total > 0 {
            let total_to_read = if total > size_of_cluster_bytes {
                size_of_cluster_bytes
            } else {
                total
            };

            let clrs = &self.private.cluster_read_stream;
            clrs.seek(starting_pos);

            clrs.read(out, total_to_read)?;

            total -= total_to_read
        }

        Ok(())
    }

    fn load_fat_directory(&self, item: &FatDirectoryItem) -> Result<FatDirectory, ErrorCode> {
        if !item.attribute.subdirectory() {
            return Err(ErrorCode::InvArg);
        }

        let cluster = item.first_cluster();
        let cluster_sector = self.cluster_to_sector(cluster);
        let total_items = get_total_items_for_directory(
            self.sector_size,
            cluster_sector,
            &self.private.directory_stream,
        )?;

        let directory_size: u16 = total_items * FatDirectoryItem::size();

        let mut items: Vec<u8> = Vec::with_capacity(directory_size.into());
        self.read_internal(cluster, 0x00, directory_size.into(), &mut items)?;
        Ok(FatDirectory {
            items: unsafe { FatDirectoryItem::from_u8_slice(items, directory_size.into()) },
            sector_pos: cluster_sector,
            ending_sector_pos: cluster_sector + (directory_size / self.sector_size),
        })
    }

    fn cluster_to_sector(&self, cluster: u16) -> u16 {
        let ending_sector_pos = self.private.root_directory.ending_sector_pos;
        let sectors_per_cluster = self.private.header.primary_header.sectors_per_cluster;
        ending_sector_pos + ((cluster - 2) * u16::from(sectors_per_cluster))
    }
}

fn get_total_items_for_directory(
    sector_size: u16,
    directory_start_sector: u16,
    directory_stream: &DiskStreamer,
) -> Result<u16, ErrorCode> {
    let mut count: u16 = 0;
    let directory_start_pos = sector_to_absolute(sector_size, directory_start_sector);

    directory_stream.seek(directory_start_pos);
    loop {
        let mut item_buf = [0; size_of::<FatDirectoryItem>()];
        let item: FatDirectoryItem = directory_stream.read_into(&mut item_buf)?;

        if item.filename[0] == BLANK_RECORD {
            break;
        }

        if item.filename[0] == UNUSED {
            continue;
        }

        count += 1;
    }

    Ok(count)
}

fn sector_to_absolute(sector_size: u16, sector: u16) -> usize {
    usize::from(sector * sector_size)
}

fn char_array_to_ascii_string(arr: &[u8]) -> Result<String, ErrorCode> {
    arr.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b.is_ascii().then_some(b as char).ok_or(ErrorCode::BadPath))
        .collect()
}

fn to_proper_fat16_bytes(
    bytes: &[u8],
    out: &mut [u8],
    size: usize,
    offset: usize,
) -> Result<usize, ErrorCode> {
    let mut i = 0;

    if size == 0 {
        return Ok(i);
    }

    for &current_byte in bytes.iter() {
        if i >= size {
            break; // We exceeded input buffer size. Cannot process anymore
        }

        if current_byte == 0x00 || current_byte == 0x20 {
            break; // We hit null or space
        }

        out[i + offset] = current_byte;
        i += 1;
    }
    Ok(i)
}

fn get_full_relative_filename(item: &FatDirectoryItem) -> Result<String, ErrorCode> {
    let mut out = [0; MAX_PATH];
    let mut offset = 0;

    offset += to_proper_fat16_bytes(&item.filename, &mut out, item.filename.len(), 0)?;

    if item.ext[0] != 0x00 && item.ext[0] != 0x20 {
        out[offset] = b'.';

        offset += 1;
        to_proper_fat16_bytes(&item.ext, &mut out, item.ext.len(), offset)?;
    }
    char_array_to_ascii_string(&out)
}

impl FileSystem for Fat16 {
    fn name(&self) -> &str {
        "FAT16"
    }

    fn fopen(
        &self,
        fd: FileDescriptorIndex,
        path: PathPart,
        mode: FileMode,
    ) -> Result<(), ErrorCode> {
        if mode != FileMode::Read {
            return Err(ErrorCode::RdOnly);
        }

        let root_directory = { Arc::clone(&self.private.root_directory) };

        let root_item = self.get_directory_entry(root_directory, path)?;

        assert!(
            self.fds.read().get(&fd).is_none(),
            "Fat16 fd {} is already assigned, but it's being requested",
            fd
        );

        self.fds.write().insert(
            fd,
            FatFileDescriptor {
                pos: 0,
                item: Arc::new(root_item),
            },
        );
        Ok(())
    }

    fn fseek(
        &self,
        fd: FileDescriptorIndex,
        offset: usize,
        whence: FileSeekMode,
    ) -> Result<(), ErrorCode> {
        let mut fds = self.fds.write();
        let descriptor = fds.get_mut(&fd).ok_or(ErrorCode::InvArg)?;

        let item = match &*descriptor.item {
            FatItem::Directory(_) => return Err(ErrorCode::InvArg),
            FatItem::File(file) => file,
        };

        // Cannot exceed max size of a fat16 file
        let offset_32: u32 = offset.try_into().map_err(|_| ErrorCode::InvArg)?;

        if offset_32 >= item.filesize {
            return Err(ErrorCode::InvArg);
        }

        match whence {
            FileSeekMode::Set => descriptor.pos = offset,
            FileSeekMode::Cur => descriptor.pos += offset,
            FileSeekMode::End => unimplemented!(),
        }
        Ok(())
    }

    fn fread(
        &self,
        out: &mut [u8],
        size: usize,
        nmemb: usize,
        fd: FileDescriptorIndex,
    ) -> Result<usize, ErrorCode> {
        let fds = self.fds.read();

        let fat_desc = fds.get(&fd).ok_or(ErrorCode::InvArg)?;

        let item_binding = Arc::clone(&fat_desc.item);
        let item = match &*item_binding {
            FatItem::File(file) => file,
            FatItem::Directory(_) => panic!("Fat16 fd {} is a directory", fd),
        };

        let offset = fat_desc.pos;

        for _ in 0..nmemb {
            self.read_internal(item.first_cluster(), offset, size, out)?;
        }
        Ok(nmemb)
    }

    fn fstat(&self, fd: FileDescriptorIndex) -> Result<FileStat, ErrorCode> {
        let fds = self.fds.read();

        let descriptor = fds.get(&fd).ok_or(ErrorCode::InvArg)?;
        let item = match &*descriptor.item {
            FatItem::Directory(_) => return Err(ErrorCode::InvArg),
            FatItem::File(file) => file,
        };

        let mut flags = FileStatFlags::default();
        if item.attribute.read_only() {
            flags.set_read_only(true);
        }

        Ok(FileStat {
            filesize: item.filesize,
            flags,
        })
    }

    fn fclose(&self, fd: FileDescriptorIndex) {
        let mut fds = self.fds.write();
        fds.remove(&fd);
    }

    fn fs_resolve(disk: &Disk) -> Result<Self, ErrorCode> {
        // Get the fat private header
        let stream: DiskStreamer = DiskStreamer::new(disk.id)?;

        let mut header_buf = [0; size_of::<FatH>()];
        let private_header: FatH = stream.read_into(&mut header_buf)?;

        let signature = private_header.extended_header.signature;

        if signature != SIGNATURE {
            return Err(ErrorCode::FsNotUs);
        }

        let directory_stream = DiskStreamer::new(disk.id)?;

        let root_directory: FatDirectory =
            FatDirectory::get_root(disk, &private_header, &directory_stream)?;

        let fat_private = FatPrivate::new(disk, private_header, root_directory, directory_stream)?;

        Ok(Self {
            private: fat_private,
            fds: RwLock::new(HashMap::new()),
            sector_size: disk.sector_size,
        })
    }
}
