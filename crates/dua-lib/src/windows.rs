//! Windows filesystem enumeration support.

use std::{
    ffi::OsString,
    io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_HANDLE_EOF, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
    },
    Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_BOTH_DIR_INFO, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_STANDARD_INFO, FileAttributeTagInfo, FileIdBothDirectoryInfo, FileStandardInfo,
        GetFileInformationByHandle, GetFileInformationByHandleEx, OPEN_EXISTING, SYNCHRONIZE,
    },
};

const DIRECTORY_BUFFER_WORDS: usize = 8 * 1024;
const NAME_SURROGATE_BIT: u32 = 0x2000_0000;

/// A filesystem entry produced by the Windows directory walker.
pub struct Entry {
    /// Distance from the walk root: `0` for the root, `1` for its children, and so on.
    pub depth: usize,
    /// File name relative to `parent_path`.
    pub file_name: OsString,
    /// Filesystem entry type without following symbolic links.
    pub file_type: FileType,
    /// Metadata obtained with directory enumeration.
    pub metadata: io::Result<Metadata>,
    /// Path containing this entry.
    pub parent_path: Arc<Path>,
}

impl Entry {
    /// Create an entry from a filesystem path.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let handle = OwnedHandle::open(
            path,
            FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            FILE_FLAG_OPEN_REPARSE_POINT,
        )?;
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: `handle` is owned and valid, and `info` is writable for the call's duration.
        if unsafe { GetFileInformationByHandle(handle.0, &raw mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut standard = FILE_STANDARD_INFO::default();
        // SAFETY: the output pointer and byte count describe the initialized `standard` value.
        if unsafe {
            GetFileInformationByHandleEx(
                handle.0,
                FileStandardInfo,
                (&raw mut standard).cast(),
                size_of::<FILE_STANDARD_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let reparse_tag = if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
            // SAFETY: the output pointer and byte count describe the initialized `tag` value.
            if unsafe {
                GetFileInformationByHandleEx(
                    handle.0,
                    FileAttributeTagInfo,
                    (&raw mut tag).cast(),
                    size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            tag.ReparseTag
        } else {
            0
        };
        let file_type = FileType::from_attributes(info.dwFileAttributes, reparse_tag);
        let file_id = u64::from(info.nFileIndexHigh) << 32 | u64::from(info.nFileIndexLow);
        let metadata = Metadata {
            len: u64::from(info.nFileSizeHigh) << 32 | u64::from(info.nFileSizeLow),
            allocated_size: standard.AllocationSize.max(0).cast_unsigned(),
            modified: filetime_to_system_time(
                u64::from(info.ftLastWriteTime.dwHighDateTime) << 32
                    | u64::from(info.ftLastWriteTime.dwLowDateTime),
            ),
            volume_serial: u64::from(info.dwVolumeSerialNumber),
            file_id: (file_id != 0).then_some(file_id),
        };
        Ok(Self {
            depth: 0,
            file_name: path.file_name().unwrap_or(path.as_os_str()).to_owned(),
            file_type,
            metadata: Ok(metadata),
            parent_path: Arc::from(path.parent().unwrap_or(Path::new(""))),
        })
    }

    /// Return the full path to this entry.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.parent_path.join(&self.file_name)
    }
}

#[derive(Clone, Copy)]
/// Windows filesystem entry type derived from directory attributes and reparse tags.
pub struct FileType {
    is_dir: bool,
    is_symlink: bool,
}

impl FileType {
    fn from_attributes(attributes: u32, reparse_tag: u32) -> Self {
        let is_symlink =
            attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 && reparse_tag & NAME_SURROGATE_BIT != 0;
        Self {
            is_dir: !is_symlink && attributes & FILE_ATTRIBUTE_DIRECTORY != 0,
            is_symlink,
        }
    }

    /// Return whether this entry is a directory that may be traversed.
    #[must_use]
    pub fn is_dir(self) -> bool {
        self.is_dir
    }

    /// Return whether this entry is a regular file.
    #[must_use]
    pub fn is_file(self) -> bool {
        !self.is_dir && !self.is_symlink
    }

    /// Return whether this entry is a symbolic link or junction.
    #[must_use]
    pub fn is_symlink(self) -> bool {
        self.is_symlink
    }
}

#[derive(Clone, Copy)]
/// Metadata supplied by Windows directory enumeration.
pub struct Metadata {
    len: u64,
    allocated_size: u64,
    modified: SystemTime,
    volume_serial: u64,
    file_id: Option<u64>,
}

impl Metadata {
    /// Return the logical file size.
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Return the number of allocated bytes.
    #[must_use]
    pub fn allocated_size(&self) -> u64 {
        self.allocated_size
    }

    #[allow(clippy::unnecessary_wraps)]
    /// Return the last modification time.
    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(self.modified)
    }

    /// Return the volume and file identifiers used to recognize hard links.
    #[must_use]
    pub fn hard_link_id(&self) -> Option<(u64, u64)> {
        self.file_id.map(|file_id| (self.volume_serial, file_id))
    }
}

pub(crate) struct ReadDir {
    handle: OwnedHandle,
    buffer: Vec<u64>,
    next_offset: Option<usize>,
    exhausted: bool,
    parent_path: Arc<Path>,
    depth: usize,
    volume_serial: u64,
}

impl ReadDir {
    pub(crate) fn open(path: Arc<Path>, depth: usize) -> io::Result<Self> {
        let handle = OwnedHandle::open(&path, FILE_LIST_DIRECTORY | SYNCHRONIZE, 0)?;
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: `handle` is owned and valid, and `info` is writable for the call's duration.
        if unsafe { GetFileInformationByHandle(handle.0, &raw mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            handle,
            buffer: vec![0; DIRECTORY_BUFFER_WORDS],
            next_offset: None,
            exhausted: false,
            parent_path: path,
            depth,
            volume_serial: u64::from(info.dwVolumeSerialNumber),
        })
    }

    fn refill(&mut self) -> io::Result<bool> {
        // SAFETY: the aligned buffer is writable for its full reported byte length, and the owned
        // directory handle remains valid while `self` lives.
        let result = unsafe {
            GetFileInformationByHandleEx(
                self.handle.0,
                FileIdBothDirectoryInfo,
                self.buffer.as_mut_ptr().cast(),
                (self.buffer.len() * size_of::<u64>()) as u32,
            )
        };
        if result != 0 {
            self.next_offset = Some(0);
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        if matches!(
            error.raw_os_error().map(i32::cast_unsigned),
            Some(ERROR_NO_MORE_FILES | ERROR_HANDLE_EOF)
        ) {
            self.exhausted = true;
            Ok(false)
        } else {
            self.exhausted = true;
            Err(error)
        }
    }

    fn entry_at(&self, offset: usize) -> io::Result<(Entry, Option<usize>)> {
        const HEADER_SIZE: usize = std::mem::offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);
        let buffer_len = self.buffer.len() * size_of::<u64>();
        if offset > buffer_len.saturating_sub(HEADER_SIZE) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Windows directory record header exceeds its buffer",
            ));
        }
        // SAFETY: validation above proves the record header lies within the aligned backing
        // allocation. Individual fields are read unaligned because record offsets need not be.
        let info_ptr = unsafe {
            self.buffer
                .as_ptr()
                .byte_add(offset)
                .cast::<FILE_ID_BOTH_DIR_INFO>()
        };
        // SAFETY: `info_ptr` addresses a complete in-bounds header validated above.
        let info = unsafe { info_ptr.read_unaligned() };
        if info.FileNameLength % 2 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Windows directory record has an odd UTF-16 byte length",
            ));
        }
        let name_len = (info.FileNameLength / 2) as usize;
        let name_bytes = name_len.checked_mul(size_of::<u16>()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Windows filename is too large")
        })?;
        if offset
            .checked_add(HEADER_SIZE)
            .and_then(|start| start.checked_add(name_bytes))
            .is_none_or(|end| end > buffer_len)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Windows directory filename exceeds its buffer",
            ));
        }
        // SAFETY: the filename range was validated against the buffer above. It may be unaligned,
        // so each UTF-16 code unit is copied with `read_unaligned` below.
        let name_ptr = unsafe { (&raw const (*info_ptr).FileName).cast::<u16>() };
        let name = (0..name_len)
            // SAFETY: `idx` stays within the validated filename range.
            .map(|idx| unsafe { name_ptr.add(idx).read_unaligned() })
            .collect::<Vec<_>>();
        let file_name = OsString::from_wide(&name);
        let next_offset = if info.NextEntryOffset == 0 {
            None
        } else {
            let next = offset
                .checked_add(info.NextEntryOffset as usize)
                .filter(|next| *next <= buffer_len.saturating_sub(HEADER_SIZE))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Windows directory record offset exceeds its buffer",
                    )
                })?;
            Some(next)
        };
        let file_type = FileType::from_attributes(info.FileAttributes, info.EaSize);
        let file_id = info.FileId.cast_unsigned();
        let file_id = (file_id != 0).then_some(file_id);
        Ok((
            Entry {
                depth: self.depth,
                file_name,
                file_type,
                metadata: Ok(Metadata {
                    len: info.EndOfFile.max(0).cast_unsigned(),
                    allocated_size: info.AllocationSize.max(0).cast_unsigned(),
                    modified: filetime_to_system_time(info.LastWriteTime.max(0).cast_unsigned()),
                    volume_serial: self.volume_serial,
                    file_id,
                }),
                parent_path: Arc::clone(&self.parent_path),
            },
            next_offset,
        ))
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.exhausted {
                return None;
            }
            if self.next_offset.is_none() {
                match self.refill() {
                    Ok(true) => {}
                    Ok(false) => return None,
                    Err(err) => return Some(Err(err)),
                }
            }
            let offset = self.next_offset.take().expect("refill provides an offset");
            match self.entry_at(offset) {
                Ok((entry, next_offset)) => {
                    self.next_offset = next_offset;
                    if entry.file_name == "." || entry.file_name == ".." {
                        continue;
                    }
                    return Some(Ok(entry));
                }
                Err(err) => {
                    self.exhausted = true;
                    return Some(Err(err));
                }
            }
        }
    }
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn open(path: &Path, access: u32, extra_flags: u32) -> io::Result<Self> {
        let path = absolute_verbatim_path(path)?;
        // SAFETY: `path` is null-terminated and all remaining pointer arguments follow the
        // `CreateFileW` contract. A successful handle is exclusively owned by the return value.
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                access,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | extra_flags,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }
}

fn absolute_verbatim_path(path: &Path) -> io::Result<Vec<u16>> {
    const SEP: u16 = b'\\' as u16;
    const ALT_SEP: u16 = b'/' as u16;
    const VERBATIM_PREFIX: &[u16] = &[SEP, SEP, b'?' as u16, SEP];
    const NT_PREFIX: &[u16] = &[SEP, b'?' as u16, b'?' as u16, SEP];
    const DEVICE_PREFIX: &[u16] = &[SEP, SEP, b'.' as u16, SEP];

    let encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    let mut verbatim = if encoded.starts_with(VERBATIM_PREFIX) || encoded.starts_with(NT_PREFIX) {
        encoded
    } else {
        let absolute = absolute_without_name_normalization(path)?;
        let mut encoded = absolute.as_os_str().encode_wide().collect::<Vec<_>>();
        for unit in &mut encoded {
            if *unit == ALT_SEP {
                *unit = SEP;
            }
        }
        if encoded.starts_with(DEVICE_PREFIX) {
            r"\\?\"
                .encode_utf16()
                .chain(encoded.into_iter().skip(4))
                .collect()
        } else if encoded.starts_with(&[SEP, SEP]) {
            r"\\?\UNC\"
                .encode_utf16()
                .chain(encoded.into_iter().skip(2))
                .collect()
        } else {
            r"\\?\".encode_utf16().chain(encoded).collect()
        }
    };
    verbatim.push(0);
    Ok(verbatim)
}

fn absolute_without_name_normalization(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        let joined = std::env::current_dir()?.join(path);
        if joined.is_absolute() {
            joined
        } else {
            return std::path::absolute(path);
        }
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: `OwnedHandle` is created only from a successful `CreateFileW` call and closes
        // that handle exactly once here.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn filetime_to_system_time(ticks: u64) -> SystemTime {
    const WINDOWS_TO_UNIX_TICKS: u64 = 116_444_736_000_000_000;
    if ticks >= WINDOWS_TO_UNIX_TICKS {
        UNIX_EPOCH + duration_from_ticks(ticks - WINDOWS_TO_UNIX_TICKS)
    } else {
        UNIX_EPOCH - duration_from_ticks(WINDOWS_TO_UNIX_TICKS - ticks)
    }
}

fn duration_from_ticks(ticks: u64) -> Duration {
    Duration::new(ticks / 10_000_000, ((ticks % 10_000_000) * 100) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_flags_match_windows_symlink_metadata_semantics() {
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
        const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;
        const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
        const IO_REPARSE_TAG_CLOUD: u32 = 0x9000_001A;

        let file = FileType::from_attributes(FILE_ATTRIBUTE_NORMAL, 0);
        assert!(!file.is_dir());
        assert!(file.is_file());
        assert!(!file.is_symlink());
        let dir = FileType::from_attributes(FILE_ATTRIBUTE_DIRECTORY, 0);
        assert!(dir.is_dir());
        assert!(!dir.is_file());
        assert!(!dir.is_symlink());
        for tag in [IO_REPARSE_TAG_SYMLINK, IO_REPARSE_TAG_MOUNT_POINT] {
            let link = FileType::from_attributes(
                FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT,
                tag,
            );
            assert!(!link.is_dir());
            assert!(!link.is_file());
            assert!(link.is_symlink());
        }
        let cloud = FileType::from_attributes(
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT,
            IO_REPARSE_TAG_CLOUD,
        );
        assert!(cloud.is_dir());
        assert!(!cloud.is_symlink());
    }

    #[test]
    fn raw_directory_entries_include_sizes_and_file_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file");
        std::fs::write(&path, b"content").unwrap();
        let mut entries = ReadDir::open(Arc::from(dir.path()), 1).unwrap();
        let entry = entries.find_map(Result::ok).unwrap();
        let metadata = entry.metadata.unwrap();
        let direct = Entry::from_path(&path).unwrap().metadata.unwrap();
        assert_eq!(metadata.len(), 7);
        assert!(metadata.allocated_size() >= metadata.len());
        assert!(metadata.hard_link_id().is_some());
        assert_eq!(metadata.len(), direct.len());
        assert_eq!(metadata.allocated_size(), direct.allocated_size());
        assert_eq!(metadata.hard_link_id(), direct.hard_link_id());
    }

    #[test]
    fn raw_directory_metadata_matches_handle_metadata_for_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("child");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("file"), b"content").unwrap();

        let entry = ReadDir::open(Arc::from(dir.path()), 1)
            .unwrap()
            .find_map(Result::ok)
            .unwrap();
        let metadata = entry.metadata.unwrap();
        let direct = Entry::from_path(&path).unwrap().metadata.unwrap();
        assert_eq!(metadata.len(), direct.len());
        assert_eq!(metadata.allocated_size(), direct.allocated_size());
        assert_eq!(metadata.hard_link_id(), direct.hard_link_id());
    }

    #[test]
    fn root_directory_symlinks_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        let link = dir.path().join("link");
        std::fs::create_dir(&target).unwrap();
        match std::os::windows::fs::symlink_dir(&target, &link) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("directory symlink can be created: {err}"),
        }

        let entry = Entry::from_path(&link).unwrap();
        assert!(entry.file_type.is_symlink());
        assert!(!entry.file_type.is_dir());
    }

    #[test]
    fn raw_directory_entries_identify_hard_links() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("original");
        std::fs::write(&original, b"content").unwrap();
        std::fs::hard_link(&original, dir.path().join("link")).unwrap();

        let ids = ReadDir::open(Arc::from(dir.path()), 1)
            .unwrap()
            .map(|entry| entry.unwrap().metadata.unwrap().hard_link_id().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], ids[1]);
    }

    #[test]
    fn raw_directory_iteration_refills_its_buffer() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..400 {
            let name = format!("{idx:04}-{}", "x".repeat(100));
            std::fs::write(dir.path().join(name), []).unwrap();
        }

        let count = ReadDir::open(Arc::from(dir.path()), 1)
            .unwrap()
            .map(Result::unwrap)
            .count();
        assert_eq!(count, 400);
    }

    #[test]
    fn raw_directory_opens_paths_longer_than_max_path() {
        let dir = tempfile::tempdir().unwrap();
        let mut path = dir.path().to_owned();
        while path.as_os_str().encode_wide().count() <= 260 {
            path.push("x".repeat(50));
            std::fs::create_dir(&path).unwrap();
        }
        std::fs::write(path.join("file"), b"content").unwrap();

        let entries = ReadDir::open(Arc::from(path), 1)
            .unwrap()
            .map(|entry| entry.unwrap().file_name)
            .collect::<Vec<_>>();
        assert_eq!(entries, [OsString::from("file")]);
    }

    #[test]
    fn raw_directory_preserves_verbatim_name_semantics() {
        let dir = tempfile::tempdir().unwrap();
        let mut verbatim_root = OsString::from(r"\\?\");
        verbatim_root.push(dir.path());
        let verbatim_child = PathBuf::from(verbatim_root).join("child.");
        std::fs::create_dir(&verbatim_child).unwrap();
        std::fs::write(verbatim_child.join("file"), b"content").unwrap();

        let ordinary_child = dir.path().join("child.");
        assert!(
            absolute_verbatim_path(&ordinary_child)
                .unwrap()
                .ends_with(&"child.\0".encode_utf16().collect::<Vec<_>>())
        );
        let entries = ReadDir::open(Arc::from(ordinary_child), 1)
            .unwrap()
            .map(|entry| entry.unwrap().file_name)
            .collect::<Vec<_>>();
        assert_eq!(entries, [OsString::from("file")]);
    }

    #[test]
    fn raw_directory_resolves_parent_components_before_using_verbatim_paths() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file"), b"content").unwrap();
        let path = dir.path().join("missing").join("..");

        let entries = ReadDir::open(Arc::from(path), 1)
            .unwrap()
            .map(|entry| entry.unwrap().file_name)
            .collect::<Vec<_>>();
        assert_eq!(entries, [OsString::from("file")]);
    }
}
