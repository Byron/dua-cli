use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct InodeFilter {
    inner: HashMap<(u64, u64), u64>,
}

impl InodeFilter {
    #[cfg(unix)]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.add_dev_inode((metadata.dev(), metadata.ino()), metadata.nlink())
    }

    #[cfg(windows)]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        use std::os::windows::fs::MetadataExt;

        if let (Some(dev), Some(inode), Some(nlinks)) = (
            metadata.volume_serial_number(),
            metadata.file_index(),
            metadata.number_of_links(),
        ) {
            self.add_dev_inode((dev as u64, inode), nlinks as u64)
        } else {
            true
        }
    }

    #[cfg(not(any(unix, windows)))]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        true
    }

    pub fn add_dev_inode(&mut self, dev_inode: (u64, u64), nlinks: u64) -> bool {
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
