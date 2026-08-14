use std::collections::HashMap;

// Darwin's `nlink_t` is 16 bits, so this marker cannot collide with an actual link count.
#[cfg(target_os = "macos")]
const UNRESOLVED_DIRECTORY_LINKS: u64 = u64::MAX;

/// Tracks seen `(device, inode)` pairs to avoid double-counting hard-linked files.
#[derive(Debug, Default, Clone)]
pub(crate) struct InodeFilter {
    inner: HashMap<(u64, u64), u64>,
}

impl InodeFilter {
    #[cfg(unix)]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(
        &mut self,
        entry: &crate::walk::Entry,
        metadata: &crate::walk::Metadata,
    ) -> bool {
        #[cfg(not(target_os = "macos"))]
        use std::os::unix::fs::MetadataExt;

        #[cfg(target_os = "macos")]
        if entry.file_type.is_dir() {
            return self.add_directory(entry, metadata);
        }

        #[cfg(not(target_os = "macos"))]
        let _ = entry;

        self.add_dev_inode((metadata.dev(), metadata.ino()), metadata.nlink())
    }

    #[cfg(windows)]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(
        &mut self,
        _entry: &crate::walk::Entry,
        metadata: &crate::walk::Metadata,
    ) -> bool {
        metadata
            .hard_link_id()
            .is_none_or(|id| self.inner.insert(id, 0).is_none())
    }

    #[cfg(not(any(unix, windows)))]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(
        &mut self,
        _entry: &crate::walk::Entry,
        metadata: &std::fs::Metadata,
    ) -> bool {
        true
    }

    #[cfg(target_os = "macos")]
    fn add_directory(
        &mut self,
        entry: &crate::walk::Entry,
        metadata: &crate::walk::Metadata,
    ) -> bool {
        let dev_inode = (metadata.dev(), metadata.ino());
        let link_count = metadata.nlink();

        match self.inner.get(&dev_inode).copied() {
            None if link_count <= 1 => {
                // ATTR_DIR_LINKCOUNT does not include the synthetic links reported by stat.
                // Resolve the actual count only if an overlapping root revisits this directory.
                self.inner.insert(dev_inode, UNRESOLVED_DIRECTORY_LINKS);
                true
            }
            None => self.add_dev_inode(dev_inode, link_count),
            Some(UNRESOLVED_DIRECTORY_LINKS) => {
                let nlinks = if link_count > 1 {
                    link_count
                } else {
                    use std::os::unix::fs::MetadataExt;

                    std::fs::symlink_metadata(entry.path())
                        .ok()
                        .filter(|actual| (actual.dev(), actual.ino()) == dev_inode)
                        .map_or(link_count, |actual| actual.nlink())
                };

                self.inner.remove(&dev_inode);
                if nlinks <= 1 {
                    return true;
                }

                // The first observation already contributed its size. Consume the current
                // observation through the ordinary counter to retain its reset behavior.
                self.inner.insert(dev_inode, nlinks - 1);
                self.add_dev_inode(dev_inode, nlinks)
            }
            Some(remaining) => self.add_dev_inode(dev_inode, remaining + 1),
        }
    }

    /// Register a `(device, inode)` with its hard-link count.
    ///
    /// Returns `true` for the first observation that should contribute to size/count,
    /// and `false` for subsequent links.
    #[cfg(any(unix, test))]
    pub(crate) fn add_dev_inode(&mut self, dev_inode: (u64, u64), nlinks: u64) -> bool {
        if nlinks <= 1 {
            return true;
        }

        match self.inner.get_mut(&dev_inode) {
            Some(1) => {
                self.inner.remove(&dev_inode);
                false
            }
            Some(count) => {
                *count -= 1;
                false
            }
            None => {
                self.inner.insert(dev_inode, nlinks - 1);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_filters_inodes() {
        let mut inodes = InodeFilter::default();

        assert!(inodes.add_dev_inode((1, 1), 2));
        assert!(inodes.add_dev_inode((2, 1), 2));
        assert!(!inodes.add_dev_inode((1, 1), 2));
        assert!(!inodes.add_dev_inode((2, 1), 2));

        assert!(inodes.add_dev_inode((1, 1), 3));
        assert!(!inodes.add_dev_inode((1, 1), 3));
        assert!(!inodes.add_dev_inode((1, 1), 3));

        assert!(inodes.add_dev_inode((1, 1), 1));
        assert!(inodes.add_dev_inode((1, 1), 1));
    }
}
