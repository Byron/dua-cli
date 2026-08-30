//! Streaming codec for dua traversal snapshots.

use crate::traverse::{EntryData, Traversal, TreeIndex};
use anyhow::{Context, Result, anyhow, bail};
use std::{
    borrow::Cow,
    collections::HashSet,
    io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAGIC: &[u8; 8] = b"DUASNAP\0";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 12;
const MAX_NAME_LEN: usize = 1024 * 1024;
const MAX_RECORD_LEN: usize = 2 * 1024 * 1024;
const DIGEST_LEN: usize = 32;

/// Encoding 0 stores Unix `OsStr` bytes verbatim.
#[cfg(unix)]
const PATH_ENCODING: u8 = 0;
/// Encoding 1 stores Windows `OsStr` wide units as little-endian `u16`s.
#[cfg(windows)]
const PATH_ENCODING: u8 = 1;

const FLAG_DIRECTORY: u8 = 0x01;
const FLAG_METADATA_IO_ERROR: u8 = 0x02;
const FLAG_ENTRY_COUNT: u8 = 0x04;
const KNOWN_FLAGS: u8 = FLAG_DIRECTORY | FLAG_METADATA_IO_ERROR | FLAG_ENTRY_COUNT;

/// A verified traversal snapshot and its input roots in their original order.
#[derive(Debug)]
pub struct Snapshot {
    /// The reconstructed traversal tree.
    pub traversal: Traversal,
    /// Top-level tree nodes in original input order.
    pub roots: Vec<TreeIndex>,
}

pub(crate) struct DecodedEntry<'a> {
    /// Zero-based depth in the serialized forest; top-level roots have depth zero.
    pub(crate) depth: usize,
    pub(crate) data: EntryData,
    /// Lossless platform-native bytes borrowed from the decoder's reusable record buffer.
    pub(crate) native_name: &'a [u8],
    pub(crate) sibling_ordinal: u64,
}

impl<'a> DecodedEntry<'a> {
    pub(crate) fn name(&self) -> Cow<'a, Path> {
        native_name_from_bytes(self.native_name)
    }
}

struct DecodeSummary {
    total_size: u128,
    total_entries: u64,
    digest: [u8; DIGEST_LEN],
}

struct OpenNode {
    id: u64,
    is_dir: bool,
    has_child: bool,
    last_child_ordinal: u64,
}

enum SnapshotReader<R> {
    Raw(BufReader<R>),
    Zlib {
        reader: BufReader<R>,
        decompressor: gix::zlib::Decompress,
        /// Remembers `StreamEnd` across reads so final output can be returned before the next read
        /// verifies that no trailing compressed input remains.
        finished: bool,
    },
}

struct Decoder<R> {
    reader: HashingReader<BufReader<SnapshotReader<R>>>,
    open_nodes: Vec<OpenNode>,
    sibling_names: Vec<Vec<u8>>,
    record: Vec<u8>,
    node_count: u64,
    total_size: u128,
    total_entries: u64,
    summary: Option<DecodeSummary>,
}

impl<R: Read> SnapshotReader<R> {
    fn new(reader: R) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        if is_zlib_header(reader.fill_buf()?) {
            Ok(Self::Zlib {
                reader,
                decompressor: gix::zlib::Decompress::new(),
                finished: false,
            })
        } else {
            Ok(Self::Raw(reader))
        }
    }
}

impl<R: Read> Read for SnapshotReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Raw(reader) => reader.read(buffer),
            Self::Zlib {
                reader,
                decompressor,
                finished,
            } => {
                if buffer.is_empty() {
                    return Ok(0);
                }
                if *finished {
                    return if reader.fill_buf()?.is_empty() {
                        Ok(0)
                    } else {
                        Err(invalid_data("compressed snapshot has trailing data"))
                    };
                }
                loop {
                    let (status, consumed, written, eof) = {
                        let input = reader.fill_buf()?;
                        let eof = input.is_empty();
                        let input_before = decompressor.total_in();
                        let output_before = decompressor.total_out();
                        let flush = if eof {
                            gix::zlib::FlushDecompress::Finish
                        } else {
                            gix::zlib::FlushDecompress::None
                        };
                        let status =
                            decompressor
                                .decompress(input, buffer, flush)
                                .map_err(|err| {
                                    io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        format!("could not decompress snapshot: {err}"),
                                    )
                                })?;
                        (
                            status,
                            (decompressor.total_in() - input_before) as usize,
                            (decompressor.total_out() - output_before) as usize,
                            eof,
                        )
                    };
                    reader.consume(consumed);

                    match status {
                        gix::zlib::Status::StreamEnd => {
                            *finished = true;
                            if written == 0 && !reader.fill_buf()?.is_empty() {
                                return Err(invalid_data("compressed snapshot has trailing data"));
                            }
                            return Ok(written);
                        }
                        gix::zlib::Status::Ok | gix::zlib::Status::BufError => {
                            if written != 0 {
                                return Ok(written);
                            }
                            if eof {
                                return Err(invalid_data("compressed snapshot is truncated"));
                            }
                            if consumed == 0 {
                                return Err(invalid_data(
                                    "compressed snapshot decoder made no progress",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_zlib_header(bytes: &[u8]) -> bool {
    let [compression, flags, ..] = bytes else {
        return false;
    };
    // CM = 8 selects the DEFLATE compression method.
    compression & 0x0f == 8
        // CINFO <= 7 limits the DEFLATE window to 32 KiB.
        && compression >> 4 <= 7
        // FCHECK requires the two header bytes to form a multiple of 31.
        && u16::from_be_bytes([*compression, *flags]) % 31 == 0
        // FDICT must be clear because preset dictionaries are unsupported.
        && flags & 0x20 == 0
}

fn verify_checksum(reader: impl Read) -> Result<[u8; DIGEST_LEN]> {
    let mut reader = SnapshotReader::new(reader)?;
    let mut hash = gix::hash::hasher(gix::hash::Kind::Sha256);
    let mut buffer = vec![0; 64 * 1024];
    let mut tail = Vec::with_capacity(DIGEST_LEN + buffer.len());
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        tail.extend_from_slice(&buffer[..read]);
        if tail.len() > DIGEST_LEN {
            let hashed = tail.len() - DIGEST_LEN;
            hash.update(&tail[..hashed]);
            tail.copy_within(hashed.., 0);
            tail.truncate(DIGEST_LEN);
        }
    }
    if tail.len() != DIGEST_LEN {
        bail!("snapshot checksum is truncated");
    }
    let actual = hash.try_finalize()?;
    if actual.as_slice() != tail {
        bail!("snapshot checksum mismatch");
    }
    let mut digest = [0; DIGEST_LEN];
    digest.copy_from_slice(actual.as_slice());
    Ok(digest)
}

/// A checksum-verified, seekable snapshot that can replay entries without materializing the tree.
///
/// Construction verifies the decompressed stream's checksum without decoding its records.
/// Records are structurally validated during each replay. The backing data must remain unchanged;
/// structural errors and changes may be reported after callbacks have observed earlier entries.
pub struct Replay<R> {
    reader: R,
    start: u64,
    digest: [u8; DIGEST_LEN],
}

pub(crate) struct ReplayEntries<'a, R> {
    decoder: Decoder<&'a mut R>,
    expected_digest: [u8; DIGEST_LEN],
}

impl<R: Read> ReplayEntries<'_, R> {
    pub(crate) fn next_entry(&mut self) -> Result<Option<DecodedEntry<'_>>> {
        self.decoder.next_entry(Some(&self.expected_digest))
    }
}

impl<R: Read + Seek> Replay<R> {
    /// Verify `reader`'s checksum and prepare it for bounded-memory replay.
    pub fn new(mut reader: R) -> Result<Self> {
        let start = reader
            .stream_position()
            .context("could not determine snapshot stream position")?;
        let digest = verify_checksum(&mut reader)?;
        // Let's keep this for safety, even though it's redundant.
        reader
            .seek(SeekFrom::Start(start))
            .context("could not rewind snapshot")?;
        Ok(Self {
            reader,
            start,
            digest,
        })
    }

    pub(crate) fn for_each_entry(
        &mut self,
        mut on_entry: impl for<'entry> FnMut(DecodedEntry<'entry>) -> Result<()>,
    ) -> Result<()> {
        let mut entries = self.entries()?;
        while let Some(entry) = entries.next_entry()? {
            on_entry(entry)?;
        }
        Ok(())
    }

    pub(crate) fn entries(&mut self) -> Result<ReplayEntries<'_, R>> {
        self.reader
            .seek(SeekFrom::Start(self.start))
            .context("could not rewind snapshot")?;
        Ok(ReplayEntries {
            decoder: Decoder::new(&mut self.reader)?,
            expected_digest: self.digest,
        })
    }
}

/// Write `traversal` as a deterministic version-1 snapshot, optionally compressed with zlib.
///
/// `roots` must contain the traversal's top-level nodes in original input order.
pub fn write(
    writer: impl Write,
    traversal: &Traversal,
    roots: &[TreeIndex],
    compression_level: Option<i32>,
) -> Result<()> {
    let Some(level) = compression_level else {
        return write_raw(writer, traversal, roots);
    };
    let compression = gix::zlib::Compression::new(level)
        .with_context(|| format!("snapshot compression level {level} is outside 0..=9"))?;
    let mut writer = gix::zlib::stream::deflate::Write::new(writer, compression);
    write_raw(&mut writer, traversal, roots)?;
    Ok(())
}

fn write_raw(writer: impl Write, traversal: &Traversal, roots: &[TreeIndex]) -> Result<()> {
    let mut writer = gix::hash::io::Write::new(BufWriter::new(writer), gix::hash::Kind::Sha256);
    let mut header = [0; HEADER_LEN];
    header[..MAGIC.len()].copy_from_slice(MAGIC);
    header[8..10].copy_from_slice(&VERSION.to_le_bytes());
    header[10] = PATH_ENCODING;
    writer.write_all(&header)?;

    let mut seen_roots = HashSet::with_capacity(roots.len());
    for &root in roots {
        if !seen_roots.insert(root) {
            bail!("snapshot roots contain duplicate node {root:?}");
        }
    }
    drop(seen_roots);

    let mut stack = Vec::new();
    let mut record = Vec::new();
    let mut node_count = 0u64;
    for &root in roots {
        stack.push((root, traversal.root_index, 0));
        while let Some((index, expected_parent, parent_id)) = stack.pop() {
            if index == traversal.root_index {
                bail!("snapshot traversal contains a cycle or shared node at {index:?}");
            }
            let entry = traversal
                .tree
                .entry(index)
                .ok_or_else(|| anyhow!("snapshot node {index:?} does not exist"))?;
            if traversal.tree.parent(index) != Some(expected_parent) {
                bail!("snapshot traversal contains a cycle or shared node at {index:?}");
            }
            let name = traversal
                .tree
                .native_name(index)
                .expect("existing tree entry has a name");
            validate_name(name, parent_id == 0)?;
            let node_id = node_count
                .checked_add(1)
                .context("snapshot contains too many nodes")?;
            let parent_distance = node_id
                .checked_sub(parent_id)
                .filter(|distance| *distance != 0)
                .context("snapshot parent was not written before its child")?;

            record.clear();
            push_uleb128(&mut record, u128::from(parent_distance));
            let mut flags = (u8::from(entry.is_dir) * FLAG_DIRECTORY)
                | (u8::from(entry.metadata_io_error) * FLAG_METADATA_IO_ERROR);
            if entry.entry_count.is_some() {
                flags |= FLAG_ENTRY_COUNT;
            }
            record.push(flags);
            push_uleb128(
                &mut record,
                u128::try_from(name.len()).context("snapshot name is too long")?,
            );
            record.extend_from_slice(name);
            push_uleb128(&mut record, entry.size);
            let (seconds, nanos) = split_time(entry.mtime)?;
            push_uleb128(&mut record, u128::from(zigzag_encode(seconds)));
            push_uleb128(&mut record, u128::from(nanos));
            if let Some(count) = entry.entry_count {
                push_uleb128(&mut record, u128::from(count));
            }
            if record.len() > MAX_RECORD_LEN {
                bail!("snapshot record exceeds the {MAX_RECORD_LEN}-byte limit");
            }
            write_uleb128(&mut writer, record.len() as u128)?;
            writer.write_all(&record)?;

            let mut children = traversal.tree.children(index).collect::<Vec<_>>();
            for &child in &children {
                validate_name(
                    traversal
                        .tree
                        .native_name(child)
                        .ok_or_else(|| anyhow!("snapshot child {child:?} does not exist"))?,
                    false,
                )?;
            }
            if !children.is_empty() && !entry.is_dir {
                bail!("snapshot file node {index:?} has children");
            }
            children.sort_by(|left, right| {
                traversal
                    .tree
                    .native_name(*left)
                    .cmp(&traversal.tree.native_name(*right))
                    .then_with(|| left.index().cmp(&right.index()))
            });
            stack.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|child| (child, index, node_id)),
            );
            node_count = node_id;
        }
    }

    writer.write_all(&[0])?;
    write_uleb128(&mut writer, u128::from(node_count))?;
    let gix::hash::io::Write { mut inner, hash } = writer;
    let digest = hash.try_finalize()?;
    inner.write_all(digest.as_slice())?;
    inner.flush()?;
    Ok(())
}

/// Read and fully verify a version-1 snapshot before returning its traversal.
pub fn read(reader: impl Read) -> Result<Snapshot> {
    let mut traversal = Traversal::new();
    traversal.cost = Some(Duration::ZERO);
    let mut parents = Vec::new();
    let mut roots = Vec::new();
    let summary = decode(reader, |entry| {
        parents.truncate(entry.depth);
        let parent = parents.last().copied().unwrap_or(traversal.root_index);
        let node = traversal
            .tree
            .try_add_child_native(parent, entry.native_name, entry.data)
            .map_err(|err| anyhow!("could not add snapshot entry: {err}"))?;
        if entry.depth == 0 {
            roots
                .try_reserve(1)
                .context("could not grow snapshot root table")?;
            roots.push(node);
        }
        parents
            .try_reserve(1)
            .context("could not grow snapshot ancestor stack")?;
        parents.push(node);
        Ok(())
    })?;

    traversal
        .tree
        .update(traversal.root_index, |synthetic_root| {
            synthetic_root.size = summary.total_size;
            synthetic_root.entry_count = (!roots.is_empty()).then_some(summary.total_entries);
        });

    Ok(Snapshot { traversal, roots })
}

fn decode(
    reader: impl Read,
    mut on_entry: impl for<'entry> FnMut(DecodedEntry<'entry>) -> Result<()>,
) -> Result<DecodeSummary> {
    let mut decoder = Decoder::new(reader)?;
    while let Some(entry) = decoder.next_entry(None)? {
        on_entry(entry)?;
    }
    Ok(decoder.summary.expect("end of snapshot stores its summary"))
}

impl<R: Read> Decoder<R> {
    fn new(reader: R) -> Result<Self> {
        let mut reader = HashingReader::new(BufReader::new(SnapshotReader::new(reader)?));
        let mut header = [0; HEADER_LEN];
        read_exact(&mut reader, &mut header)?;
        if &header[..MAGIC.len()] != MAGIC {
            bail!("invalid snapshot at byte 0: bad magic");
        }
        let version = u16::from_le_bytes([header[8], header[9]]);
        if version != VERSION {
            bail!("invalid snapshot at byte 8: unsupported version {version}");
        }
        if header[10] != PATH_ENCODING {
            bail!(
                "invalid snapshot at byte 10: path encoding {} is incompatible with this host",
                header[10]
            );
        }
        if header[11] != 0 {
            bail!("invalid snapshot at byte 11: unknown header flags");
        }

        Ok(Self {
            reader,
            open_nodes: Vec::new(),
            sibling_names: Vec::new(),
            record: Vec::new(),
            node_count: 0,
            total_size: 0,
            total_entries: 0,
            summary: None,
        })
    }

    fn next_entry(
        &mut self,
        expected_digest: Option<&[u8; DIGEST_LEN]>,
    ) -> Result<Option<DecodedEntry<'_>>> {
        if self.summary.is_some() {
            return Ok(None);
        }

        let length_offset = self.reader.offset;
        let record_len = read_u64(&mut self.reader)
            .map_err(|err| anyhow!("invalid snapshot integer at byte {length_offset}: {err}"))?;
        if record_len == 0 {
            self.finish()?;
            if expected_digest.is_some_and(|expected| {
                self.summary
                    .as_ref()
                    .expect("end of snapshot stores its summary")
                    .digest
                    != *expected
            }) {
                bail!("snapshot changed since it was verified");
            }
            return Ok(None);
        }
        let record_len = usize::try_from(record_len)
            .context("snapshot record length exceeds this address space")?;
        if record_len > MAX_RECORD_LEN {
            bail!(
                "invalid snapshot at byte {length_offset}: record exceeds the {MAX_RECORD_LEN}-byte limit"
            );
        }

        self.record.clear();
        if self.record.capacity() < record_len {
            self.record
                .try_reserve(record_len)
                .context("could not allocate snapshot record")?;
        }
        self.record.resize(record_len, 0);
        let record_offset = self.reader.offset;
        read_exact(&mut self.reader, &mut self.record)?;

        let node_id = self
            .node_count
            .checked_add(1)
            .context("snapshot contains too many nodes")?;
        let (parent_id, native_name, data) = parse_record(&self.record, node_id)
            .map_err(|err| anyhow!("invalid snapshot record at byte {record_offset}: {err}"))?;
        if node_id >= u64::from(u32::MAX) {
            bail!("snapshot exceeds the tree node-index limit");
        }

        let sibling_ordinal = if parent_id == 0 {
            self.open_nodes.clear();
            0
        } else {
            while self
                .open_nodes
                .last()
                .is_some_and(|node| node.id != parent_id)
            {
                self.open_nodes.pop();
            }
            let parent_depth = self.open_nodes.len().saturating_sub(1);
            let parent = self.open_nodes.last_mut().with_context(|| {
                format!(
                    "invalid snapshot record at byte {record_offset}: parent is outside the current depth-first subtree"
                )
            })?;
            if !parent.is_dir {
                bail!("invalid snapshot record at byte {record_offset}: parent is not a directory");
            }
            let previous = &mut self.sibling_names[parent_depth];
            let ordinal = if parent.has_child && previous.as_slice() > native_name {
                bail!(
                    "invalid snapshot record at byte {record_offset}: sibling names are not in canonical order"
                );
            } else if parent.has_child && previous.as_slice() == native_name {
                parent
                    .last_child_ordinal
                    .checked_add(1)
                    .context("snapshot contains too many duplicate sibling names")?
            } else {
                0
            };
            previous.clear();
            previous.extend_from_slice(native_name);
            parent.has_child = true;
            parent.last_child_ordinal = ordinal;
            ordinal
        };

        let depth = self.open_nodes.len();
        if depth == 0 {
            self.total_size = self
                .total_size
                .checked_add(data.size)
                .context("snapshot root size total overflows u128")?;
            self.total_entries = self
                .total_entries
                .checked_add(data.entry_count.unwrap_or(1))
                .context("snapshot root entry count total overflows u64")?;
        }

        self.open_nodes
            .try_reserve(1)
            .context("could not grow snapshot ancestor stack")?;
        if self.sibling_names.len() <= depth {
            self.sibling_names
                .try_reserve(1)
                .context("could not grow snapshot sibling buffers")?;
            self.sibling_names.push(Vec::new());
        } else {
            self.sibling_names[depth].clear();
        }
        self.open_nodes.push(OpenNode {
            id: node_id,
            is_dir: data.is_dir,
            has_child: false,
            last_child_ordinal: 0,
        });
        self.node_count = node_id;
        Ok(Some(DecodedEntry {
            depth,
            data,
            native_name,
            sibling_ordinal,
        }))
    }

    fn finish(&mut self) -> Result<()> {
        let count_offset = self.reader.offset;
        let footer_count = read_u64(&mut self.reader)
            .map_err(|err| anyhow!("invalid snapshot node count at byte {count_offset}: {err}"))?;
        if footer_count != self.node_count {
            bail!(
                "invalid snapshot at byte {count_offset}: footer names {footer_count} nodes but read {}",
                self.node_count
            );
        }

        let actual = self.reader.hash.clone().try_finalize()?;
        let mut expected = [0; DIGEST_LEN];
        self.reader
            .inner
            .read_exact(&mut expected)
            .context("snapshot checksum is truncated")?;
        if actual.as_slice() != expected {
            bail!("snapshot checksum mismatch");
        }
        let mut trailing = [0];
        if self.reader.inner.read(&mut trailing)? != 0 {
            bail!("snapshot has trailing data");
        }

        let mut digest = [0; DIGEST_LEN];
        digest.copy_from_slice(actual.as_slice());
        self.summary = Some(DecodeSummary {
            total_size: self.total_size,
            total_entries: self.total_entries,
            digest,
        });
        Ok(())
    }
}

fn parse_record(record: &[u8], node_id: u64) -> Result<(u64, &[u8], EntryData)> {
    let mut cursor = io::Cursor::new(record);
    let parent_distance = read_u64(&mut cursor)?;
    let parent_id = node_id
        .checked_sub(parent_distance)
        .filter(|_| parent_distance != 0)
        .context("parent distance is zero or points forward")?;

    let mut flags = [0];
    cursor.read_exact(&mut flags)?;
    let flags = flags[0];
    if flags & !KNOWN_FLAGS != 0 {
        bail!("unknown node flags {flags:#04x}");
    }

    let name_len = usize::try_from(read_u64(&mut cursor)?)
        .context("snapshot name length exceeds this address space")?;
    if name_len > MAX_NAME_LEN {
        bail!("name exceeds the {MAX_NAME_LEN}-byte limit");
    }
    let name_start = usize::try_from(cursor.position()).context("record offset is too large")?;
    let name_end = name_start
        .checked_add(name_len)
        .filter(|end| *end <= record.len())
        .context("record ends within its name")?;
    let name = &record[name_start..name_end];
    cursor.set_position(name_end as u64);
    validate_name(name, parent_id == 0)?;

    let size = read_uleb128(&mut cursor, 128)?;
    let seconds = zigzag_decode(read_u64(&mut cursor)?);
    let nanos = u32::try_from(read_u64(&mut cursor)?)
        .ok()
        .filter(|nanos| *nanos < 1_000_000_000)
        .context("modification time has invalid nanoseconds")?;
    let mtime = join_time(seconds, nanos)?;
    let entry_count = if flags & FLAG_ENTRY_COUNT != 0 {
        Some(read_u64(&mut cursor)?)
    } else {
        None
    };
    if cursor.position() != record.len() as u64 {
        bail!("record length does not match its fields");
    }

    Ok((
        parent_id,
        name,
        EntryData {
            size,
            mtime,
            entry_count,
            metadata_io_error: flags & FLAG_METADATA_IO_ERROR != 0,
            is_dir: flags & FLAG_DIRECTORY != 0,
        },
    ))
}

fn split_time(time: SystemTime) -> Result<(i64, u32)> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok((
            i64::try_from(duration.as_secs())
                .context("modification time is outside the snapshot range")?,
            duration.subsec_nanos(),
        )),
        Err(error) => {
            let duration = error.duration();
            let nanos = duration.subsec_nanos();
            let seconds = if nanos == 0 {
                -i128::from(duration.as_secs())
            } else {
                -i128::from(duration.as_secs()) - 1
            };
            Ok((
                i64::try_from(seconds)
                    .context("modification time is outside the snapshot range")?,
                if nanos == 0 { 0 } else { 1_000_000_000 - nanos },
            ))
        }
    }
}

fn join_time(seconds: i64, nanos: u32) -> Result<SystemTime> {
    let time = if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::new(seconds.cast_unsigned(), nanos))
    } else if nanos == 0 {
        UNIX_EPOCH.checked_sub(Duration::new(seconds.unsigned_abs(), 0))
    } else {
        UNIX_EPOCH.checked_sub(Duration::new(
            seconds.unsigned_abs() - 1,
            1_000_000_000 - nanos,
        ))
    };
    let time = time.context("modification time is not representable on this host")?;
    if split_time(time)? != (seconds, nanos) {
        bail!("modification time loses precision on this host");
    }
    Ok(time)
}

fn zigzag_encode(value: i64) -> u64 {
    (value.cast_unsigned() << 1) ^ (value >> 63).cast_unsigned()
}

fn zigzag_decode(value: u64) -> i64 {
    (value >> 1).cast_signed() ^ -(value & 1).cast_signed()
}

fn push_uleb128(out: &mut Vec<u8>, mut value: u128) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        out.push(byte | (u8::from(value != 0) * 0x80));
        if value == 0 {
            return;
        }
    }
}

fn write_uleb128(out: &mut impl Write, value: u128) -> io::Result<()> {
    let mut encoded = [0; 19];
    let mut len = 0;
    let mut value = value;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        encoded[len] = byte | (u8::from(value != 0) * 0x80);
        len += 1;
        if value == 0 {
            return out.write_all(&encoded[..len]);
        }
    }
}

fn read_u64(input: &mut impl Read) -> io::Result<u64> {
    u64::try_from(read_uleb128(input, 64)?).map_err(|_| invalid_data("ULEB128 value overflows u64"))
}

fn read_uleb128(input: &mut impl Read, bits: u32) -> io::Result<u128> {
    let groups = bits.div_ceil(7);
    let mut value = 0u128;
    for group in 0..groups {
        let mut byte = [0];
        input.read_exact(&mut byte)?;
        let payload = u128::from(byte[0] & 0x7f);
        let shift = group * 7;
        let remaining = bits - shift;
        if remaining < 7 && payload >= 1u128 << remaining {
            return Err(invalid_data("ULEB128 value overflows its field"));
        }
        value |= payload << shift;
        if byte[0] & 0x80 == 0 {
            if group != 0 && payload == 0 {
                return Err(invalid_data("ULEB128 value is not canonical"));
            }
            return Ok(value);
        }
    }
    Err(invalid_data("ULEB128 value is overlong"))
}

#[cfg(unix)]
fn native_name_from_bytes(name: &[u8]) -> Cow<'_, Path> {
    use std::os::unix::ffi::OsStrExt as _;
    Cow::Borrowed(Path::new(std::ffi::OsStr::from_bytes(name)))
}

#[cfg(all(test, unix))]
fn native_name_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(all(test, windows))]
fn native_name_bytes(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt as _;
    path.as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(windows)]
fn native_name_from_bytes(name: &[u8]) -> Cow<'_, Path> {
    use std::os::windows::ffi::OsStringExt as _;
    debug_assert_eq!(name.len() % 2, 0);
    let wide = name
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect::<Vec<_>>();
    Cow::Owned(std::path::PathBuf::from(std::ffi::OsString::from_wide(
        &wide,
    )))
}

#[cfg(unix)]
fn validate_name(name: &[u8], is_root: bool) -> Result<()> {
    if name.contains(&0) {
        bail!("path contains NUL");
    }
    if !is_root && (name.is_empty() || name.contains(&b'/') || matches!(name, b"." | b"..")) {
        bail!("child name is not one native path component");
    }
    if name.len() > MAX_NAME_LEN {
        bail!("name exceeds the {MAX_NAME_LEN}-byte limit");
    }
    Ok(())
}

#[cfg(windows)]
fn validate_name(name: &[u8], is_root: bool) -> Result<()> {
    if name.len() % 2 != 0 {
        bail!("Windows path has an odd byte length");
    }
    let mut wide = name
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
    if wide.clone().any(|unit| unit == 0) {
        bail!("path contains NUL");
    }
    if !is_root {
        let first = wide.next();
        let second = wide.next();
        if first.is_none()
            || name.chunks_exact(2).any(|bytes| {
                let unit = u16::from_le_bytes([bytes[0], bytes[1]]);
                unit == u16::from(b'/') || unit == u16::from(b'\\')
            })
            || first == Some(u16::from(b'.')) && second.is_none()
            || first == Some(u16::from(b'.'))
                && second == Some(u16::from(b'.'))
                && wide.next().is_none()
        {
            bail!("child name is not one native path component");
        }
    }
    if name.len() > MAX_NAME_LEN {
        bail!("name exceeds the {MAX_NAME_LEN}-byte limit");
    }
    Ok(())
}

struct HashingReader<R> {
    inner: R,
    hash: gix::hash::Hasher,
    offset: u64,
}

impl<R> HashingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hash: gix::hash::hasher(gix::hash::Kind::Sha256),
            offset: 0,
        }
    }
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.hash.update(&buffer[..read]);
        self.offset = self
            .offset
            .checked_add(read as u64)
            .ok_or_else(|| invalid_data("snapshot byte offset overflow"))?;
        Ok(read)
    }
}

fn read_exact<R: Read>(reader: &mut HashingReader<R>, buffer: &mut [u8]) -> Result<()> {
    reader
        .read_exact(buffer)
        .with_context(|| format!("snapshot is truncated at byte {}", reader.offset))
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests;
