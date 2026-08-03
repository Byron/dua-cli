use std::collections::HashMap;

/// Tracks seen `(device, inode)` pairs to avoid double-counting hard-linked files.
#[derive(Debug, Default, Clone)]
pub(crate) struct InodeFilter {
    inner: HashMap<(u64, u64), u64>,
}

impl InodeFilter {
    #[cfg(unix)]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.add_dev_inode((metadata.dev(), metadata.ino()), metadata.nlink())
    }

    #[cfg(windows)]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(&mut self, metadata: &crate::walk::Metadata) -> bool {
        metadata
            .hard_link_id()
            .is_none_or(|id| self.inner.insert(id, 0).is_none())
    }

    #[cfg(not(any(unix, windows)))]
    /// Register file metadata and return `true` if this link should be counted.
    pub(crate) fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        true
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
