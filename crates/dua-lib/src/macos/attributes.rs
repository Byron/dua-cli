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
pub(super) const VBLK: u32 = 3;
pub(super) const VCHR: u32 = 4;
pub(super) const VLNK: u32 = 5;
pub(super) const VSOCK: u32 = 6;
pub(super) const VFIFO: u32 = 7;
// These `<sys/stat.h>` filesystem flags and accounting units are absent from `libc`.
pub(super) const SF_FIRMLINK: u32 = 0x0080_0000;
pub(super) const STAT_BLOCK_BYTES: u64 = 512;
// `<sys/stat.h>` defines this extended flag, but `libc` does not expose it.
const EF_MAY_SHARE_BLOCKS: u64 = 0x0000_0001;
// Entry-specific bulk enumeration errors are defined by `<sys/attr.h>`.
const ATTR_CMN_ERROR: libc::attrgroup_t = 0x2000_0000;
const NANOS_PER_SECOND: u32 = 1_000_000_000;

const FTS_COMMON_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_CMN_RETURNED_ATTRS
    | libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_DEVID
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_CRTIME
    | libc::ATTR_CMN_MODTIME
    | libc::ATTR_CMN_CHGTIME
    | libc::ATTR_CMN_ACCTIME
    | libc::ATTR_CMN_OWNERID
    | libc::ATTR_CMN_GRPID
    | libc::ATTR_CMN_ACCESSMASK
    | libc::ATTR_CMN_FLAGS
    | libc::ATTR_CMN_FILEID;
// XNU's `vfs_attrlist.c` `LIST_DIR_ATTRS` authorizes this subset with LIST_DIRECTORY alone.
// Metadata attributes additionally require SEARCH, which a readable directory need not grant.
const LIST_ONLY_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_CMN_RETURNED_ATTRS
    | libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_FILEID
    | ATTR_CMN_ERROR;
const FTS_FILE_ATTRIBUTES: libc::attrgroup_t = libc::ATTR_FILE_LINKCOUNT
    | libc::ATTR_FILE_ALLOCSIZE
    | libc::ATTR_FILE_IOBLOCKSIZE
    | libc::ATTR_FILE_DEVTYPE
    | libc::ATTR_FILE_DATALENGTH;
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
            FTS_COMMON_ATTRIBUTES
        },
        volattr: 0,
        dirattr: 0,
        fileattr: if list_only { 0 } else { FTS_FILE_ATTRIBUTES },
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
    returned_common: libc::attrgroup_t,
    returned_file: libc::attrgroup_t,
    pub(super) file_name: Option<OsString>,
    pub(super) device: Option<u64>,
    pub(super) object_type: Option<u32>,
    pub(super) modified: Option<SystemTime>,
    pub(super) flags: u32,
    pub(super) inode: Option<u64>,
    pub(super) error: u32,
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

pub(super) fn parse_record(
    bytes: &[u8],
    pack_invalid: bool,
    include_apfs: bool,
) -> io::Result<ParsedRecord> {
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

    let mut record = ParsedRecord {
        returned_common: returned.commonattr,
        returned_file: returned.fileattr,
        ..Default::default()
    };

    if returned.commonattr & ATTR_CMN_ERROR != 0 {
        record.error = cursor.take_u32()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_NAME, pack_invalid) {
        // `getattrlist(2)` defines attr_dataoffset relative to this attrreference.
        let reference_offset = bytes.len() - cursor.bytes.len();
        let reference = libc::attrreference_t {
            attr_dataoffset: cursor.take_i32()?,
            attr_length: cursor.take_u32()?,
        };
        if returned.commonattr & libc::ATTR_CMN_NAME != 0 {
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
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_DEVID, pack_invalid) {
        // Darwin's signed `dev_t` is sign-extended by `std::fs::Metadata::dev()`.
        let device = i64::from(cursor.take_i32()?).cast_unsigned();
        if returned.commonattr & libc::ATTR_CMN_DEVID != 0 {
            record.device = Some(device);
        }
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_OBJTYPE, pack_invalid) {
        let object_type = cursor.take_u32()?;
        if returned.commonattr & libc::ATTR_CMN_OBJTYPE != 0 {
            record.object_type = Some(object_type);
        }
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_CRTIME, pack_invalid) {
        cursor.take_i64()?;
        cursor.take_i64()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_MODTIME, pack_invalid) {
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
        if returned.commonattr & libc::ATTR_CMN_MODTIME != 0 {
            record.modified = if seconds >= 0 {
                UNIX_EPOCH.checked_add(Duration::new(seconds.cast_unsigned(), nanos))
            } else {
                UNIX_EPOCH
                    .checked_sub(Duration::new(seconds.unsigned_abs(), 0))
                    .and_then(|time| time.checked_add(Duration::new(0, nanos)))
            };
        }
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_CHGTIME, pack_invalid) {
        cursor.take_i64()?;
        cursor.take_i64()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_ACCTIME, pack_invalid) {
        cursor.take_i64()?;
        cursor.take_i64()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_OWNERID, pack_invalid) {
        cursor.take_u32()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_GRPID, pack_invalid) {
        cursor.take_u32()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_ACCESSMASK, pack_invalid) {
        cursor.take_u32()?;
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_FLAGS, pack_invalid) {
        let flags = cursor.take_u32()?;
        if returned.commonattr & libc::ATTR_CMN_FLAGS != 0 {
            record.flags = flags;
        }
    }
    if attribute_is_packed(returned.commonattr, libc::ATTR_CMN_FILEID, pack_invalid) {
        let inode = cursor.take_u64()?;
        if returned.commonattr & libc::ATTR_CMN_FILEID != 0 {
            record.inode = Some(inode);
        }
    }

    let pack_file = pack_invalid && record.object_type != Some(VDIR);
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_LINKCOUNT, pack_file) {
        let links = u64::from(cursor.take_u32()?);
        if returned.fileattr & libc::ATTR_FILE_LINKCOUNT != 0 {
            record.file_links = Some(links);
        }
    }
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_ALLOCSIZE, pack_file) {
        let allocated = cursor.take_u64()?;
        if returned.fileattr & libc::ATTR_FILE_ALLOCSIZE != 0 {
            record.file_allocated = Some(fts_allocated_bytes(allocated));
        }
    }
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_IOBLOCKSIZE, pack_file) {
        cursor.take_u32()?;
    }
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_DEVTYPE, pack_file) {
        cursor.take_u32()?;
    }
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_DATALENGTH, pack_file) {
        let length = cursor.take_u64()?;
        if returned.fileattr & libc::ATTR_FILE_DATALENGTH != 0 {
            record.file_length = Some(length);
        }
    }
    let pack_apfs = pack_file && include_apfs;
    if attribute_is_packed(returned.fileattr, libc::ATTR_FILE_DATAALLOCSIZE, pack_apfs) {
        let allocated = cursor.take_u64()?;
        if returned.fileattr & libc::ATTR_FILE_DATAALLOCSIZE != 0 {
            record.file_data_allocated = Some(fts_allocated_bytes(allocated));
        }
    }

    if attribute_is_packed(
        returned.forkattr,
        libc::ATTR_CMNEXT_CLONEID,
        pack_invalid && include_apfs,
    ) {
        let clone_id = cursor.take_u64()?;
        if returned.forkattr & libc::ATTR_CMNEXT_CLONEID != 0 {
            record.clone_id = NonZeroU64::new(clone_id);
        }
    }
    if attribute_is_packed(
        returned.forkattr,
        libc::ATTR_CMNEXT_EXT_FLAGS,
        pack_invalid && include_apfs,
    ) {
        let extended_flags = cursor.take_u64()?;
        if returned.forkattr & libc::ATTR_CMNEXT_EXT_FLAGS != 0 {
            record.extended_flags = Some(extended_flags);
        }
    }
    Ok(record)
}

/// Return whether a requested attribute occupies space in the record.
///
/// Packed-invalid records contain default values even when the returned-attribute mask marks the
/// attribute unavailable; ordinary records omit unavailable attributes entirely.
fn attribute_is_packed(
    returned: libc::attrgroup_t,
    attribute: libc::attrgroup_t,
    pack_invalid: bool,
) -> bool {
    pack_invalid || returned & attribute != 0
}

fn fts_allocated_bytes(bytes: u64) -> u64 {
    bytes
        .div_ceil(STAT_BLOCK_BYTES)
        .saturating_mul(STAT_BLOCK_BYTES)
}

impl ParsedRecord {
    /// Return whether Apple FTS would synthesize a complete `stat` from this bulk record.
    ///
    /// FTS always stats directories, tolerates an unavailable creation time, and only requires a
    /// device type for block and character devices. Matching that validity gate keeps consumers of
    /// this walker on the same allocation-size path as macOS `du`.
    pub(super) fn satisfies_fts_stat_contract(&self) -> bool {
        let Some(object_type) = self.object_type else {
            return false;
        };
        if !matches!(object_type, VREG | VBLK | VCHR | VLNK | VSOCK | VFIFO) {
            return false;
        }

        let required_common = FTS_COMMON_ATTRIBUTES & !libc::ATTR_CMN_CRTIME;
        let mut required_file = FTS_FILE_ATTRIBUTES;
        if object_type != VBLK && object_type != VCHR {
            required_file &= !libc::ATTR_FILE_DEVTYPE;
        }

        self.returned_common & required_common == required_common
            && self.returned_file & required_file == required_file
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_record(object_type: u32) -> ParsedRecord {
        ParsedRecord {
            returned_common: FTS_COMMON_ATTRIBUTES,
            returned_file: FTS_FILE_ATTRIBUTES,
            object_type: Some(object_type),
            ..Default::default()
        }
    }

    #[test]
    fn full_request_matches_fts_without_directory_attributes() {
        let attributes = requested_attributes(false, false);

        assert_eq!(attributes.commonattr, FTS_COMMON_ATTRIBUTES);
        assert_eq!(attributes.dirattr, 0);
        assert_eq!(attributes.fileattr, FTS_FILE_ATTRIBUTES);
        assert_eq!(attributes.forkattr, 0);

        let apfs_attributes = requested_attributes(false, true);
        assert_eq!(apfs_attributes.commonattr, FTS_COMMON_ATTRIBUTES);
        assert_eq!(apfs_attributes.dirattr, 0);
        assert_eq!(
            apfs_attributes.fileattr,
            FTS_FILE_ATTRIBUTES | APFS_FILE_ATTRIBUTES
        );
        assert_eq!(apfs_attributes.forkattr, APFS_EXTENDED_ATTRIBUTES);
    }

    #[test]
    fn fts_contract_accepts_only_known_non_directory_vnode_types() {
        for object_type in [VREG, VBLK, VCHR, VLNK, VSOCK, VFIFO] {
            assert!(
                complete_record(object_type).satisfies_fts_stat_contract(),
                "known vnode type {object_type} must use complete bulk metadata"
            );
        }
        for object_type in [VNON, VDIR, 8, u32::MAX] {
            assert!(
                !complete_record(object_type).satisfies_fts_stat_contract(),
                "unknown or directory vnode type {object_type} must fall back to stat"
            );
        }

        assert!(
            !ParsedRecord {
                returned_common: FTS_COMMON_ATTRIBUTES,
                returned_file: FTS_FILE_ATTRIBUTES,
                ..Default::default()
            }
            .satisfies_fts_stat_contract(),
            "a missing vnode type must fall back to stat"
        );
    }

    #[test]
    fn fts_contract_rejects_missing_required_attributes() {
        let mut missing_common = complete_record(VREG);
        missing_common.returned_common &= !libc::ATTR_CMN_DEVID;
        assert!(!missing_common.satisfies_fts_stat_contract());

        let mut missing_file = complete_record(VREG);
        missing_file.returned_file &= !libc::ATTR_FILE_ALLOCSIZE;
        assert!(!missing_file.satisfies_fts_stat_contract());

        let mut missing_creation_time = complete_record(VREG);
        missing_creation_time.returned_common &= !libc::ATTR_CMN_CRTIME;
        assert!(missing_creation_time.satisfies_fts_stat_contract());

        let mut regular_without_device_type = complete_record(VREG);
        regular_without_device_type.returned_file &= !libc::ATTR_FILE_DEVTYPE;
        assert!(regular_without_device_type.satisfies_fts_stat_contract());

        let mut block_without_device_type = complete_record(VBLK);
        block_without_device_type.returned_file &= !libc::ATTR_FILE_DEVTYPE;
        assert!(!block_without_device_type.satisfies_fts_stat_contract());
    }

    #[test]
    fn fts_allocation_rounds_up_to_512_byte_stat_blocks() {
        assert_eq!(fts_allocated_bytes(0), 0);
        assert_eq!(fts_allocated_bytes(1), 512);
        assert_eq!(fts_allocated_bytes(512), 512);
        assert_eq!(fts_allocated_bytes(513), 1024);
    }
}
