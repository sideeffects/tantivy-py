// Ported from tantivy PR #3058 (quickwit-oss/tantivy, MIT), which adds
// ImmutableDirectory to the tantivy crate. Drop this file once that PR
// ships and tantivy-py exposes it.
// Ref: https://github.com/quickwit-oss/tantivy/pull/3058

use std::path::Path;
use std::sync::Arc;
use std::{fmt, io};

use tantivy::directory::error::{
    DeleteError, LockError, OpenReadError, OpenWriteError,
};
use tantivy::directory::{
    Directory, DirectoryLock, FileHandle, Lock, WatchCallback, WatchHandle,
    WritePtr, META_LOCK,
};

/// Forwards reads to the wrapped directory and rejects writes. `META_LOCK`
/// is a no-op and the directory is not watched, so the index must not
/// change while this wrapper is in use.
#[derive(Clone)]
pub(crate) struct ImmutableDirectory {
    directory: Box<dyn Directory>,
}

impl ImmutableDirectory {
    pub(crate) fn new(directory: impl Into<Box<dyn Directory>>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
}

impl fmt::Debug for ImmutableDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImmutableDirectory({:?})", self.directory)
    }
}

impl Directory for ImmutableDirectory {
    fn get_file_handle(
        &self,
        path: &Path,
    ) -> Result<Arc<dyn FileHandle>, OpenReadError> {
        self.directory.get_file_handle(path)
    }

    fn delete(&self, path: &Path) -> Result<(), DeleteError> {
        Err(DeleteError::IoError {
            io_error: Arc::new(immutable_error()),
            filepath: path.to_path_buf(),
        })
    }

    fn exists(&self, path: &Path) -> Result<bool, OpenReadError> {
        self.directory.exists(path)
    }

    fn open_write(&self, path: &Path) -> Result<WritePtr, OpenWriteError> {
        Err(OpenWriteError::wrap_io_error(
            immutable_error(),
            path.to_path_buf(),
        ))
    }

    fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
        self.directory.atomic_read(path)
    }

    fn atomic_write(&self, _path: &Path, _data: &[u8]) -> io::Result<()> {
        Err(immutable_error())
    }

    fn sync_directory(&self) -> io::Result<()> {
        Ok(())
    }

    fn acquire_lock(&self, lock: &Lock) -> Result<DirectoryLock, LockError> {
        if lock.filepath == META_LOCK.filepath {
            return Ok(DirectoryLock::from(Box::new(())));
        }
        Err(LockError::wrap_io_error(immutable_error()))
    }

    fn watch(
        &self,
        _watch_callback: WatchCallback,
    ) -> tantivy::Result<WatchHandle> {
        Ok(WatchHandle::empty())
    }
}

fn immutable_error() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "directory is immutable")
}
