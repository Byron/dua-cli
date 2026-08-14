//! macOS directory enumeration and metadata collection using `getattrlistbulk`.

use std::{
    ffi::OsString,
    fs, io,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

mod attributes;

use attributes::{
    AlignedBuffer, ParsedRecord, RecordHeader, SF_FIRMLINK, STAT_BLOCK_BYTES, VDIR, VLNK, VNON,
    VREG, invalid_data, parse_record, read_record_length, requested_attributes,
};

const DIRECTORY_BUFFER_BYTES: usize = 64 * 1024;

/// A macOS filesystem entry produced from native directory metadata.
pub struct Entry {
    /// Distance from the walk root: `0` for the root, `1` for its children, and so on.
    pub depth: usize,
    /// File name relative to `parent_path`.
    pub file_name: OsString,
    /// Filesystem entry type without following symbolic links.
    pub file_type: FileType,
    /// Metadata returned while enumerating this entry, or an entry-specific I/O error.
    pub metadata: io::Result<Metadata>,
    /// Path containing this entry.
    pub parent_path: Arc<Path>,
}

impl Entry {
    /// Create an entry for an explicitly requested root without following symbolic links.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = Metadata::from_std(&fs::symlink_metadata(path)?);
        Ok(Self {
            depth: 0,
            file_name: path.file_name().unwrap_or(path.as_os_str()).to_owned(),
            file_type: metadata.file_type,
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

/// macOS filesystem entry type obtained without following symbolic links.
#[derive(Clone, Copy)]
pub struct FileType {
    kind: u32,
}

impl FileType {
    fn from_std(file_type: fs::FileType) -> Self {
        let kind = if file_type.is_dir() {
            VDIR
        } else if file_type.is_file() {
            VREG
        } else if file_type.is_symlink() {
            VLNK
        } else {
            VNON
        };
        Self { kind }
    }

    /// Return whether this entry is a directory that may be traversed.
    #[must_use]
    pub fn is_dir(self) -> bool {
        self.kind == VDIR
    }

    /// Return whether this entry is a regular file.
    #[must_use]
    pub fn is_file(self) -> bool {
        self.kind == VREG
    }

    /// Return whether this entry is a symbolic link.
    #[must_use]
    pub fn is_symlink(self) -> bool {
        self.kind == VLNK
    }
}

/// macOS metadata obtained during native directory enumeration.
#[derive(Clone, Copy)]
pub struct Metadata {
    len: u64,
    allocated_size: u64,
    modified: Option<SystemTime>,
    dev: u64,
    ino: u64,
    nlink: u64,
    file_type: FileType,
}

impl Metadata {
    fn from_std(metadata: &fs::Metadata) -> Self {
        let allocated_size = metadata.blocks().saturating_mul(STAT_BLOCK_BYTES);
        Self {
            len: metadata.len(),
            allocated_size,
            modified: metadata.modified().ok(),
            dev: metadata.dev(),
            ino: metadata.ino(),
            nlink: metadata.nlink(),
            file_type: FileType::from_std(metadata.file_type()),
        }
    }

    /// Return the logical file or directory length.
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Return the number of bytes physically allocated to the file or directory.
    #[must_use]
    pub fn allocated_size(&self) -> u64 {
        self.allocated_size
    }

    /// Return the allocated size as 512-byte filesystem accounting blocks.
    #[must_use]
    pub fn blocks(&self) -> u64 {
        self.allocated_size.div_ceil(STAT_BLOCK_BYTES)
    }

    /// Return the last modification time.
    pub fn modified(&self) -> io::Result<SystemTime> {
        self.modified
            .ok_or_else(|| invalid_data("macOS modification time is unavailable"))
    }

    /// Return the device number of the filesystem containing this entry.
    #[must_use]
    pub fn dev(&self) -> u64 {
        self.dev
    }

    /// Return the filesystem inode number.
    #[must_use]
    pub fn ino(&self) -> u64 {
        self.ino
    }

    /// Return the number of hard links to this entry.
    ///
    /// Bulk-enumerated directories use `ATTR_DIR_LINKCOUNT`, which `getattrlist(2)` says
    /// excludes historical `.` and `..` links and can therefore differ from `stat(2)`.
    #[must_use]
    pub fn nlink(&self) -> u64 {
        self.nlink
    }

    /// Return whether this metadata describes a regular file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }
}

pub(crate) struct ReadDir {
    directory: OwnedFd,
    buffer: Box<AlignedBuffer<DIRECTORY_BUFFER_BYTES>>,
    offset: usize,
    remaining: usize,
    exhausted: bool,
    listing_error: Option<i32>,
    parent_path: Arc<Path>,
    depth: usize,
}

impl ReadDir {
    pub(crate) fn open(path: Arc<Path>, depth: usize) -> io::Result<Self> {
        let directory: OwnedFd = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY)
            .open(&path)?
            .into();
        Ok(Self {
            directory,
            buffer: Box::new(AlignedBuffer::new()),
            offset: 0,
            remaining: 0,
            exhausted: false,
            listing_error: None,
            parent_path: path,
            depth,
        })
    }

    fn refill(&mut self) -> io::Result<bool> {
        loop {
            let mut attributes = requested_attributes(self.listing_error.is_some());
            // SAFETY: the directory descriptor is owned and remains open, `attributes` is a valid
            // initialized Darwin attrlist, and the aligned buffer is writable for its exact size.
            let count = unsafe {
                libc::getattrlistbulk(
                    self.directory.as_raw_fd(),
                    (&raw mut attributes).cast(),
                    self.buffer.as_mut_bytes().as_mut_ptr().cast(),
                    self.buffer.as_bytes().len(),
                    0,
                )
            };
            if count > 0 {
                self.offset = 0;
                self.remaining = usize::try_from(count)
                    .map_err(|_| invalid_data("macOS directory record count is invalid"))?;
                return Ok(true);
            }
            if count == 0 {
                self.exhausted = true;
                return Ok(false);
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if self.listing_error.is_none() && error.raw_os_error() == Some(libc::EACCES) {
                self.listing_error = Some(libc::EACCES);
                continue;
            }
            self.exhausted = true;
            return Err(error);
        }
    }

    fn next_record(&mut self) -> io::Result<Entry> {
        let bytes = self.buffer.as_bytes();
        let buffer_len = bytes.len();
        if self.offset > buffer_len.saturating_sub(size_of::<u32>()) {
            self.exhausted = true;
            return Err(invalid_data("macOS directory record has no length"));
        }
        let length = read_record_length(&bytes[self.offset..])?;
        let Some(end) = self
            .offset
            .checked_add(length)
            .filter(|end| length >= size_of::<RecordHeader>() && *end <= buffer_len)
        else {
            self.exhausted = true;
            return Err(invalid_data("macOS directory record exceeds its buffer"));
        };
        let record = &bytes[self.offset..end];
        self.offset = end;
        self.remaining -= 1;

        let mut parsed = parse_record(record)?;
        let file_name = parsed
            .file_name
            .take()
            .ok_or_else(|| invalid_data("macOS directory record has no filename"))?;
        let metadata_error = if parsed.error != 0 {
            Some(parsed.error.cast_signed())
        } else {
            self.listing_error
        };
        let file_type = FileType {
            kind: parsed.object_type.unwrap_or(VNON),
        };
        let metadata = if let Some(error) = metadata_error {
            Err(io::Error::from_raw_os_error(error))
        } else {
            if parsed.object_type.is_none() {
                return Err(invalid_data("macOS directory record has no object type"));
            }
            let metadata_from_path = || {
                fs::symlink_metadata(self.parent_path.join(&file_name))
                    .map(|metadata| Metadata::from_std(&metadata))
            };
            // XNU uses NOCROSSMOUNT for bulk lookup; stat the visible mount or firmlink.
            let special_mount = parsed.flags & SF_FIRMLINK != 0
                || parsed.mount_status & libc::DIR_MNTSTATUS_MNTPOINT != 0;
            if special_mount {
                metadata_from_path()
            } else {
                parsed.metadata(file_type).or_else(|_| metadata_from_path())
            }
        };
        let file_type = metadata
            .as_ref()
            .map_or(file_type, |metadata| metadata.file_type);
        Ok(Entry {
            depth: self.depth,
            file_name,
            file_type,
            metadata,
            parent_path: Arc::clone(&self.parent_path),
        })
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.exhausted {
                return None;
            }
            if self.remaining == 0 {
                match self.refill() {
                    Ok(true) => {}
                    Ok(false) => return None,
                    Err(error) => return Some(Err(error)),
                }
            }
            match self.next_record() {
                Ok(entry) if entry.file_name == "." || entry.file_name == ".." => {}
                Ok(entry) => return Some(Ok(entry)),
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

impl ParsedRecord {
    fn metadata(&self, file_type: FileType) -> io::Result<Metadata> {
        let (len, allocated_size, nlink) = if file_type.is_dir() {
            (
                self.directory_length
                    .ok_or_else(|| invalid_data("missing directory length"))?,
                self.directory_allocated
                    .ok_or_else(|| invalid_data("missing directory allocation"))?,
                self.directory_links
                    .ok_or_else(|| invalid_data("missing directory hard-link count"))?,
            )
        } else {
            (
                self.file_length
                    .ok_or_else(|| invalid_data("missing file length"))?,
                self.file_allocated
                    .ok_or_else(|| invalid_data("missing file allocation"))?,
                self.file_links
                    .ok_or_else(|| invalid_data("missing file hard-link count"))?,
            )
        };

        Ok(Metadata {
            len,
            allocated_size,
            modified: Some(
                self.modified
                    .ok_or_else(|| invalid_data("missing modification timestamp"))?,
            ),
            dev: self
                .device
                .ok_or_else(|| invalid_data("missing device number"))?,
            ino: self
                .inode
                .ok_or_else(|| invalid_data("missing inode number"))?,
            nlink,
            file_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    #[test]
    fn directory_reader_rejects_regular_files_before_iteration() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("file");
        fs::write(&file, b"content").unwrap();

        let error = crate::read_dir(&file)
            .err()
            .expect("regular files must be rejected before iteration");
        assert_eq!(error.raw_os_error(), Some(libc::ENOTDIR));
    }

    #[test]
    fn bulk_metadata_matches_std_for_regular_files_resource_forks_and_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("file");
        fs::write(&file, b"data").unwrap();
        let data_blocks = fs::metadata(&file).unwrap().blocks();
        let resource = file.join("..namedfork/rsrc");
        fs::write(resource, [7; 8192]).unwrap();
        let resource_blocks = fs::metadata(&file).unwrap().blocks();
        assert!(
            resource_blocks > data_blocks,
            "resource fork must allocate storage: data={data_blocks}, total={resource_blocks}"
        );
        std::os::unix::fs::symlink("file", directory.path().join("link")).unwrap();

        let entries = ReadDir::open(Arc::from(directory.path()), 1)
            .unwrap()
            .collect::<io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(
            entries.len(),
            2,
            "regular file and symbolic link must both be enumerated"
        );
        for entry in entries {
            let direct = fs::symlink_metadata(entry.path()).unwrap();
            let metadata = entry.metadata.unwrap();
            assert_eq!(
                metadata.len(),
                direct.len(),
                "{:?} logical size",
                entry.file_name
            );
            assert_eq!(
                metadata.allocated_size(),
                direct.blocks() * STAT_BLOCK_BYTES,
                "{:?} allocated size",
                entry.file_name
            );
            assert_eq!(metadata.dev(), direct.dev());
            assert_eq!(metadata.ino(), direct.ino());
            assert_eq!(metadata.modified().unwrap(), direct.modified().unwrap());
            assert_eq!(
                entry.file_type.is_symlink(),
                direct.file_type().is_symlink()
            );
        }
    }

    #[test]
    fn bulk_metadata_preserves_fractional_pre_epoch_timestamps() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("file");
        let file = fs::File::create(&path).unwrap();
        let expected = std::time::UNIX_EPOCH - std::time::Duration::from_millis(900);
        file.set_times(fs::FileTimes::new().set_modified(expected))
            .unwrap();
        let direct = file.metadata().unwrap().modified().unwrap();

        let entry = ReadDir::open(Arc::from(directory.path()), 1)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();

        assert_eq!(direct, expected);
        assert_eq!(entry.metadata.unwrap().modified().unwrap(), direct);
    }

    #[test]
    fn readable_directory_without_search_permission_preserves_entry_errors() {
        let directory = tempfile::tempdir().unwrap();
        let restricted = directory.path().join("restricted");
        fs::create_dir(&restricted).unwrap();
        fs::write(restricted.join("file"), b"content").unwrap();
        fs::set_permissions(&restricted, fs::Permissions::from_mode(0o400)).unwrap();

        let entries = ReadDir::open(Arc::from(restricted.as_path()), 1)
            .unwrap()
            .collect::<Vec<_>>();
        fs::set_permissions(&restricted, fs::Permissions::from_mode(0o700)).unwrap();

        assert_eq!(entries.len(), 1);
        let entry = entries.into_iter().next().unwrap().unwrap();
        assert_eq!(entry.file_name, "file");
        assert_eq!(entry.file_type.kind, VREG);
        let error = entry
            .metadata
            .err()
            .expect("metadata must retain the directory search-permission error");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn bulk_metadata_identifies_hard_links() {
        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("original");
        fs::write(&original, [7; 8192]).unwrap();
        fs::hard_link(&original, directory.path().join("hard-link")).unwrap();

        let entries = ReadDir::open(Arc::from(directory.path()), 1)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name, entry.metadata.unwrap())
            })
            .collect::<std::collections::HashMap<_, _>>();
        let original = &entries[std::ffi::OsStr::new("original")];
        let hard_link = &entries[std::ffi::OsStr::new("hard-link")];

        assert_eq!(original.ino(), hard_link.ino());
        assert_eq!(original.nlink(), 2);
    }

    #[test]
    fn bulk_device_numbers_match_std_on_devfs() {
        let directory = Path::new("/dev");
        let entry = ReadDir::open(Arc::from(directory), 1)
            .unwrap()
            .find(|entry| entry.as_ref().is_ok_and(|entry| entry.file_name == "null"))
            .expect("devfs contains /dev/null")
            .unwrap();
        let expected = fs::symlink_metadata(directory.join("null")).unwrap();

        assert_eq!(entry.metadata.unwrap().dev(), expected.dev());
    }

    #[test]
    fn bulk_directory_reader_refills_its_buffer() {
        let directory = tempfile::tempdir().unwrap();
        for index in 0..700 {
            fs::write(
                directory
                    .path()
                    .join(format!("{index:04}-{}", "x".repeat(100))),
                [],
            )
            .unwrap();
        }

        let count = ReadDir::open(Arc::from(directory.path()), 1)
            .unwrap()
            .map(Result::unwrap)
            .count();
        assert_eq!(count, 700);
    }
}
