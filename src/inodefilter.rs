#![cfg_attr(windows, feature(windows_by_handle))]

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct InodeFilter {
    inner: HashMap<u64, u64>,
}

impl InodeFilter {
    #[cfg(unix)]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.add_inode(metadata.ino(), metadata.nlink())
    }

    #[cfg(windows)]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        use std::os::windows::fs::MetadataExt;

        if let (Some(inode), Some(nlinks)) = (metadata.file_index(), metadata.number_of_links()) {
            self.add_inode(inode, nlinks as u64)
        } else {
            true
        }
    }

    #[cfg(not(any(unix, windows)))]
    pub fn add(&mut self, metadata: &std::fs::Metadata) -> bool {
        true
    }

    pub fn add_inode(&mut self, inode: u64, nlinks: u64) -> bool {
        if nlinks <= 1 {
            return true;
        }

        match self.inner.get_mut(&inode) {
            Some(count) => {
                *count -= 1;

                if *count == 0 {
                    self.inner.remove(&inode);
                }

                false
            }
            None => {
                self.inner.insert(inode, nlinks - 1);
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

        assert!(inodes.add_inode(1, 2));
        assert!(!inodes.add_inode(1, 2));

        assert!(inodes.add_inode(1, 3));
        assert!(!inodes.add_inode(1, 3));
        assert!(!inodes.add_inode(1, 3));

        assert!(inodes.add_inode(1, 1));
        assert!(inodes.add_inode(1, 1));
    }
}
