use std::{io, path::Path};

#[cfg(unix)]
pub fn init(path: &Path) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;

    path.metadata().map(|m| m.dev())
}

#[cfg(unix)]
pub fn is_same_device(device_id: u64, meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    meta.dev() == device_id
}

#[cfg(not(any(unix, windows)))]
pub fn is_same_device(device_id: u64, meta: &std::fs::Metadata) -> bool {
    true
}

#[cfg(not(any(unix, windows)))]
pub fn init(path: &Path) -> io::Result<u64> {
    Ok(0)
}
