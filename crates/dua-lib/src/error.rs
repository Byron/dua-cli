use std::{fmt, io, path::Path, sync::Arc};

/// A traversal failure and the filesystem path that caused it.
#[derive(Debug)]
pub struct Error {
    path: Option<Arc<Path>>,
    source: io::Error,
}

impl Error {
    pub(crate) fn at_path(source: io::Error, path: impl Into<Arc<Path>>) -> Self {
        Self {
            path: Some(path.into()),
            source,
        }
    }

    /// Return the failing entry's path, or its containing directory when enumeration
    /// failed before an entry name was available. Paths are not canonicalized.
    ///
    /// Returns `None` for worker failures or supplied errors without path information.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Return the original I/O error, retaining its kind, OS code, and source.
    #[must_use]
    pub fn io_error(&self) -> &io::Error {
        &self.source
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            write!(f, "{}: ", path.display())?;
        }
        self.source.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self { path: None, source }
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> Self {
        Self::new(error.source.kind(), error)
    }
}
