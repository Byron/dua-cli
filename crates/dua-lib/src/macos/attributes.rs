//! Safe decoding of Darwin's packed filesystem attribute records.

use std::{
    ffi::OsString,
    io,
    num::NonZeroU64,
    os::unix::ffi::OsStringExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

// Darwin's vnode types are declared by `<sys/vnode.h>` but omitted by `libc`.
pub(super) const VNON: u32 = 0;
pub(super) const VREG: u32 = 1;
pub(super) const VDIR: u32 = 2;
pub(super) const VLNK: u32 = 5;
// These `<sys/stat.h>` filesystem flags and accounting units are absent from `libc`.
pub(super) const SF_FIRMLINK: u32 = 0x0080_0000;
pub(super) const STAT_BLOCK_BYTES: u64 = 512;
// `<sys/stat.h>` defines this extended flag, but `libc` does not expose it.
const EF_MAY_SHARE_BLOCKS: u64 = 0x0000_0001;
// Entry-specific bulk enumeration errors are defined by `<sys/attr.h>`.
const ATTR_CMN_ERROR: libc::attrgroup_t = 0x2000_0000;
const NANOS_PER_SECOND: u32 = 1_000_000_000;

const COMMON_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_CMN_RETURNED_ATTRS
    | libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_DEVID
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_MODTIME
    | libc::ATTR_CMN_FLAGS
    | libc::ATTR_CMN_FILEID
    | ATTR_CMN_ERROR;
// XNU's `vfs_attrlist.c` `LIST_DIR_ATTRS` authorizes this subset with LIST_DIRECTORY alone.
// Metadata attributes additionally require SEARCH, which a readable directory need not grant.
const LIST_ONLY_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_CMN_RETURNED_ATTRS
    | libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_FILEID
    | ATTR_CMN_ERROR;
const DIRECTORY_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_DIR_LINKCOUNT
    | libc::ATTR_DIR_MOUNTSTATUS
    | libc::ATTR_DIR_ALLOCSIZE
    | libc::ATTR_DIR_DATALENGTH;
const FILE_ATTRIBUTES: libc::attrgroup_t =
    libc::ATTR_FILE_LINKCOUNT | libc::ATTR_FILE_ALLOCSIZE | libc::ATTR_FILE_DATALENGTH;
const APFS_FILE_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_FILE_DATAALLOCSIZE;
const APFS_EXTENDED_ATTRIBUTES: libc::attrgroup_t =
    libc::ATTR_CMNEXT_CLONEID | libc::ATTR_CMNEXT_EXT_FLAGS;

/// `getattrlistbulk(2)` requires each returned record to begin on an eight-byte boundary.
#[repr(align(8))]
pub(super) struct AlignedBuffer<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> AlignedBuffer<BYTES> {
    pub(super) fn new() -> Self {
        Self([0; BYTES])
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub(super) fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Fixed record prefix; `getattrlist(2)` packs subsequent attributes on four-byte boundaries.
#[repr(C)]
pub(super) struct RecordHeader {
    length: u32,
    returned: libc::attribute_set_t,
}

/// Capacity and alignment of the explicit-root response, without overlaying its packed bytes.
#[repr(C)]
pub(super) struct RootCloneResponse {
    header: RecordHeader,
    device: libc::dev_t,
    inode: u64,
    data_allocated: u64,
    clone_id: u64,
    extended_flags: u64,
}

#[derive(Clone, Copy)]
pub(super) struct DataFork {
    pub(super) allocated_size: u64,
    pub(super) clone_id: Option<NonZeroU64>,
}

pub(super) fn requested_attributes(list_only: bool, include_apfs: bool) -> libc::attrlist {
    let mut attributes = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: if list_only {
            LIST_ONLY_ATTRIBUTES
        } else {
            COMMON_ATTRIBUTES
        },
        volattr: 0,
        dirattr: if list_only { 0 } else { DIRECTORY_ATTRIBUTES },
        fileattr: if list_only { 0 } else { FILE_ATTRIBUTES },
        forkattr: 0,
    };
    if include_apfs && !list_only {
        attributes.fileattr |= APFS_FILE_ATTRIBUTES;
        attributes.forkattr = APFS_EXTENDED_ATTRIBUTES;
    }
    attributes
}

pub(super) fn root_clone_attributes() -> libc::attrlist {
    libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: libc::ATTR_CMN_RETURNED_ATTRS | libc::ATTR_CMN_DEVID | libc::ATTR_CMN_FILEID,
        volattr: 0,
        dirattr: 0,
        fileattr: APFS_FILE_ATTRIBUTES,
        forkattr: APFS_EXTENDED_ATTRIBUTES,
    }
}

#[derive(Default)]
pub(super) struct ParsedRecord {
    pub(super) file_name: Option<OsString>,
    pub(super) device: Option<u64>,
    pub(super) object_type: Option<u32>,
    pub(super) modified: Option<SystemTime>,
    pub(super) flags: u32,
    pub(super) inode: Option<u64>,
    pub(super) error: u32,
    pub(super) directory_links: Option<u64>,
    pub(super) mount_status: u32,
    pub(super) directory_allocated: Option<u64>,
    pub(super) directory_length: Option<u64>,
    pub(super) file_links: Option<u64>,
    pub(super) file_length: Option<u64>,
    pub(super) file_allocated: Option<u64>,
    file_data_allocated: Option<u64>,
    clone_id: Option<NonZeroU64>,
    extended_flags: Option<u64>,
}

impl ParsedRecord {
    /// Clone identity is meaningful only when independent data-fork allocation is also known.
    pub(super) fn data_fork(&self) -> Option<DataFork> {
        let allocated_size = self.file_data_allocated?;
        let clone_id = self
            .extended_flags
            .filter(|flags| flags & EF_MAY_SHARE_BLOCKS != 0)
            .and(self.clone_id);
        Some(DataFork {
            allocated_size,
            clone_id,
        })
    }
}

pub(super) fn read_record_length(bytes: &[u8]) -> io::Result<usize> {
    usize::try_from(Cursor::new(bytes).take_u32()?)
        .map_err(|_| invalid_data("macOS attribute record length exceeds the address space"))
}

pub(super) fn parse_record(bytes: &[u8]) -> io::Result<ParsedRecord> {
    if bytes.len() < size_of::<RecordHeader>() {
        return Err(invalid_data("macOS attribute record is truncated"));
    }

    let mut cursor = Cursor::new(bytes);
    let _length = cursor.take_u32()?;
    let returned = libc::attribute_set_t {
        commonattr: cursor.take_u32()?,
        volattr: cursor.take_u32()?,
        dirattr: cursor.take_u32()?,
        fileattr: cursor.take_u32()?,
        forkattr: cursor.take_u32()?,
    };
    if returned.commonattr & libc::ATTR_CMN_RETURNED_ATTRS == 0 {
        return Err(invalid_data("macOS attributes omit their returned bitmap"));
    }

    let mut record = ParsedRecord::default();

    if returned.commonattr & ATTR_CMN_ERROR != 0 {
        record.error = cursor.take_u32()?;
    }
    if returned.commonattr & libc::ATTR_CMN_NAME != 0 {
        // `getattrlist(2)` defines attr_dataoffset relative to this attrreference.
        let reference_offset = bytes.len() - cursor.bytes.len();
        let reference = libc::attrreference_t {
            attr_dataoffset: cursor.take_i32()?,
            attr_length: cursor.take_u32()?,
        };
        let offset = isize::try_from(reference.attr_dataoffset)
            .map_err(|_| invalid_data("macOS filename has an invalid offset"))?;
        let start = reference_offset
            .checked_add_signed(offset)
            .ok_or_else(|| invalid_data("macOS filename has an invalid offset"))?;
        let length = usize::try_from(reference.attr_length)
            .map_err(|_| invalid_data("macOS filename exceeds the address space"))?;
        let end = start
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid_data("macOS filename exceeds its record"))?;
        let mut name = &bytes[start..end];
        if name.last() == Some(&0) {
            name = &name[..name.len() - 1];
        }
        record.file_name = Some(OsString::from_vec(name.to_vec()));
    }
    if returned.commonattr & libc::ATTR_CMN_DEVID != 0 {
        // Darwin's signed `dev_t` is sign-extended by `std::fs::Metadata::dev()`.
        record.device = Some(i64::from(cursor.take_i32()?).cast_unsigned());
    }
    if returned.commonattr & libc::ATTR_CMN_OBJTYPE != 0 {
        record.object_type = Some(cursor.take_u32()?);
    }
    if returned.commonattr & libc::ATTR_CMN_MODTIME != 0 {
        let seconds = cursor.take_i64()?;
        let nanoseconds = cursor.take_i64()?;
        // Apple represents fractional pre-epoch timestamps with negative nanoseconds.
        // Normalize them exactly as Rust's Unix `SystemTime` implementation does.
        let (seconds, nanoseconds) = if seconds <= 0
            && seconds > i64::MIN
            && nanoseconds < 0
            && nanoseconds > -i64::from(NANOS_PER_SECOND)
        {
            (seconds - 1, nanoseconds + i64::from(NANOS_PER_SECOND))
        } else {
            (seconds, nanoseconds)
        };
        let nanos = u32::try_from(nanoseconds)
            .ok()
            .filter(|nanos| *nanos < NANOS_PER_SECOND)
            .ok_or_else(|| invalid_data("macOS modification time has invalid nanoseconds"))?;
        record.modified = if seconds >= 0 {
            UNIX_EPOCH.checked_add(Duration::new(seconds.cast_unsigned(), nanos))
        } else {
            UNIX_EPOCH
                .checked_sub(Duration::new(seconds.unsigned_abs(), 0))
                .and_then(|time| time.checked_add(Duration::new(0, nanos)))
        };
    }
    if returned.commonattr & libc::ATTR_CMN_FLAGS != 0 {
        record.flags = cursor.take_u32()?;
    }
    if returned.commonattr & libc::ATTR_CMN_FILEID != 0 {
        record.inode = Some(cursor.take_u64()?);
    }

    if returned.dirattr & libc::ATTR_DIR_LINKCOUNT != 0 {
        record.directory_links = Some(u64::from(cursor.take_u32()?));
    }
    if returned.dirattr & libc::ATTR_DIR_MOUNTSTATUS != 0 {
        record.mount_status = cursor.take_u32()?;
    }
    if returned.dirattr & libc::ATTR_DIR_ALLOCSIZE != 0 {
        record.directory_allocated = Some(cursor.take_u64()?);
    }
    if returned.dirattr & libc::ATTR_DIR_DATALENGTH != 0 {
        record.directory_length = Some(cursor.take_u64()?);
    }

    if returned.fileattr & libc::ATTR_FILE_LINKCOUNT != 0 {
        record.file_links = Some(u64::from(cursor.take_u32()?));
    }
    if returned.fileattr & libc::ATTR_FILE_ALLOCSIZE != 0 {
        record.file_allocated = Some(cursor.take_u64()?);
    }
    if returned.fileattr & libc::ATTR_FILE_DATALENGTH != 0 {
        record.file_length = Some(cursor.take_u64()?);
    }
    if returned.fileattr & libc::ATTR_FILE_DATAALLOCSIZE != 0 {
        record.file_data_allocated = Some(cursor.take_u64()?);
    }

    if returned.forkattr & libc::ATTR_CMNEXT_CLONEID != 0 {
        record.clone_id = NonZeroU64::new(cursor.take_u64()?);
    }
    if returned.forkattr & libc::ATTR_CMNEXT_EXT_FLAGS != 0 {
        record.extended_flags = Some(cursor.take_u64()?);
    }
    Ok(record)
}

struct Cursor<'a> {
    bytes: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn take<const BYTES: usize>(&mut self) -> io::Result<[u8; BYTES]> {
        let (value, remaining) = self
            .bytes
            .split_first_chunk::<BYTES>()
            .ok_or_else(|| invalid_data("macOS attribute exceeds its record"))?;
        self.bytes = remaining;
        Ok(*value)
    }

    fn take_u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_ne_bytes(self.take()?))
    }

    fn take_i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_ne_bytes(self.take()?))
    }

    fn take_u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_ne_bytes(self.take()?))
    }

    fn take_i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_ne_bytes(self.take()?))
    }
}

pub(super) fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
