//! macOS directory enumeration and metadata collection using `getattrlistbulk`.

use std::{
    ffi::{CString, OsString},
    fs, io,
    num::NonZeroU64,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

mod attributes;

use attributes::{
    AlignedBuffer, DataFork, ParsedRecord, RecordHeader, RootCloneResponse, SF_FIRMLINK,
    STAT_BLOCK_BYTES, VDIR, VLNK, VNON, VREG, invalid_data, parse_record, read_record_length,
    requested_attributes, root_clone_attributes,
};

const DIRECTORY_BUFFER_BYTES: usize = 64 * 1024;

/// A macOS filesystem entry produced from native directory metadata.
pub struct Entry {
    /// Distance from the walk root: `0` for the root, `1` and highger for its children.
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
    pub fn from_path(path: &Path, options: crate::Options) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        let data_fork =
            if options.apfs_clone_metadata && metadata.is_file() && metadata.blocks() != 0 {
                clone_attributes_at(path, &metadata)
            } else {
                None
            };
        let metadata = Metadata::from_std(&metadata, data_fork);
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
    /// Data-fork allocation and clone identity, present only when extended metadata was requested
    /// and supported by the filesystem. Unlike `ino`, which identifies hard links to one file,
    /// the clone identity can be shared by copy-on-write clones with distinct inodes.
    data_fork: Option<DataFork>,
    modified: Option<SystemTime>,
    dev: u64,
    ino: u64,
    nlink: u64,
    file_type: FileType,
}

impl Metadata {
    fn from_std(metadata: &fs::Metadata, data_fork: Option<DataFork>) -> Self {
        let allocated_size = metadata.blocks().saturating_mul(STAT_BLOCK_BYTES);
        let file_type = FileType::from_std(metadata.file_type());
        Self {
            len: metadata.len(),
            allocated_size,
            data_fork: data_fork
                .filter(|fork| file_type.is_file() && fork.allocated_size <= allocated_size),
            modified: metadata.modified().ok(),
            dev: metadata.dev(),
            ino: metadata.ino(),
            nlink: metadata.nlink(),
            file_type,
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

    /// Return data-fork allocation, or total allocation when separate fork metadata is unavailable.
    #[must_use]
    pub fn data_allocated_size(&self) -> u64 {
        self.data_fork
            .map_or(self.allocated_size, |fork| fork.allocated_size)
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

    /// Return the shared APFS content identifier for a file that may have full clones.
    ///
    /// Clone identifiers are meaningful only within the same filesystem device.
    #[must_use]
    pub fn clone_id(&self) -> Option<NonZeroU64> {
        self.data_fork.and_then(|fork| fork.clone_id)
    }
}

pub(crate) struct ReadDir {
    directory: OwnedFd,
    fallback: Option<fs::ReadDir>,
    buffer: Box<AlignedBuffer<DIRECTORY_BUFFER_BYTES>>,
    offset: usize,
    remaining: usize,
    exhausted: bool,
    /// Whether bulk reads request APFS extended attributes; disabled when they cannot be served.
    /// Note that this makes the call more expensive.
    extended_attributes: bool,
    listing_error: Option<i32>,
    parent_path: Arc<Path>,
    depth: usize,
}

impl ReadDir {
    pub(crate) fn open(path: Arc<Path>, depth: usize, options: crate::Options) -> io::Result<Self> {
        let directory: OwnedFd = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY)
            .open(&path)?
            .into();
        Ok(Self {
            directory,
            fallback: None,
            buffer: Box::new(AlignedBuffer::new()),
            offset: 0,
            remaining: 0,
            exhausted: false,
            extended_attributes: options.apfs_clone_metadata,
            listing_error: None,
            parent_path: path,
            depth,
        })
    }

    /// Refill the bulk-record buffer or activate ordinary directory iteration when the filesystem
    /// does not support bulk enumeration.
    ///
    /// Returns `true` when iteration can continue, either because bulk records are available or
    /// because `self.fallback` was initialized. Returns `false` when the directory is exhausted.
    fn refill(&mut self) -> io::Result<bool> {
        loop {
            let mut attributes =
                requested_attributes(self.listing_error.is_some(), self.extended_attributes);
            // SAFETY: the directory descriptor is owned and remains open, `attributes` is a valid
            // initialized Darwin attrlist, and the aligned buffer is writable for its exact size.
            let count = unsafe {
                libc::getattrlistbulk(
                    self.directory.as_raw_fd(),
                    (&raw mut attributes).cast(),
                    self.buffer.as_mut_bytes().as_mut_ptr().cast(),
                    self.buffer.as_bytes().len(),
                    if self.extended_attributes {
                        u64::from(libc::FSOPT_ATTR_CMN_EXTENDED)
                    } else {
                        0
                    },
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
                self.extended_attributes = false;
                continue;
            }
            if self.extended_attributes
                && error.raw_os_error().is_some_and(|errno| {
                    [libc::EINVAL, libc::ENOTSUP, libc::EOPNOTSUPP, libc::ENOSYS].contains(&errno)
                })
            {
                self.extended_attributes = false;
                continue;
            }
            if error.kind() == io::ErrorKind::Unsupported
                || error.raw_os_error() == Some(libc::ENOTSUP)
                || error.raw_os_error() == Some(libc::EOPNOTSUPP)
            {
                match fs::read_dir(&self.parent_path) {
                    Ok(entries) => {
                        self.fallback = Some(entries);
                        return Ok(true);
                    }
                    Err(error) => {
                        self.exhausted = true;
                        return Err(error);
                    }
                }
            }
            self.exhausted = true;
            return Err(error);
        }
    }

    fn fallback_entry(&self, entry: fs::DirEntry) -> Entry {
        let file_name = entry.file_name();
        let metadata =
            fs::symlink_metadata(entry.path()).map(|metadata| Metadata::from_std(&metadata, None));
        let file_type = metadata.as_ref().map_or_else(
            |_| {
                entry
                    .file_type()
                    .map_or(FileType { kind: VNON }, FileType::from_std)
            },
            |metadata| metadata.file_type,
        );
        Entry {
            depth: self.depth,
            file_name,
            file_type,
            metadata,
            parent_path: Arc::clone(&self.parent_path),
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
                    .map(|metadata| Metadata::from_std(&metadata, None))
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
            if let Some(fallback) = &mut self.fallback {
                let entry = fallback.next()?;
                return Some(entry.map(|entry| self.fallback_entry(entry)));
            }
            if self.exhausted {
                return None;
            }
            if self.remaining == 0 {
                match self.refill() {
                    Ok(true) => continue,
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

fn clone_attributes_at(path: &Path, metadata: &fs::Metadata) -> Option<DataFork> {
    let path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut attributes = root_clone_attributes();
    let mut buffer = AlignedBuffer::<{ size_of::<RootCloneResponse>() }>::new();
    // SAFETY: `path` is a valid NUL-terminated C string, `attributes` is initialized, and the
    // aligned output array remains uniquely writable for its full reported byte length.
    let result = unsafe {
        libc::getattrlist(
            path.as_ptr(),
            (&raw mut attributes).cast(),
            buffer.as_mut_bytes().as_mut_ptr().cast(),
            buffer.as_bytes().len(),
            libc::FSOPT_ATTR_CMN_EXTENDED | libc::FSOPT_NOFOLLOW,
        )
    };
    if result != 0 {
        return None;
    }
    let bytes = buffer.as_bytes();
    let length = read_record_length(bytes).ok()?;
    let record = bytes.get(..length)?;
    let parsed = parse_record(record).ok()?;
    if parsed.device != Some(metadata.dev()) || parsed.inode != Some(metadata.ino()) {
        return None;
    }
    parsed.data_fork()
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
            data_fork: self
                .data_fork()
                .filter(|fork| file_type.is_file() && fork.allocated_size <= allocated_size),
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
mod tests;
