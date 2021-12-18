use crate::error::*;
use std::path::Path;

use filetime::{set_file_atime, set_file_mtime, FileTime};

/// Flags which can be used to configure how file and directory times will be
/// assigned.
#[derive(Clone)]
pub struct TimeOptions {
    /// Keep the same modification time.
    pub retain_modification_time: bool,
    /// Keep the same access time.
    pub retain_access_time: bool,
}

impl TimeOptions {
    /// Initialize struct TimeOptions with default value.
    pub fn new() -> Self {
        TimeOptions {
            retain_modification_time: false,
            retain_access_time: false,
        }
    }
}

/// Assign time attributes for `to` same as in `from`.
pub fn copy_time<P, Q>(from: P, to: Q, options: &TimeOptions) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    if options.retain_modification_time || options.retain_access_time {
        match from.as_ref().metadata() {
            Ok(metadata) => {
                let mtime = FileTime::from_last_modification_time(&metadata);
                let atime = FileTime::from_last_access_time(&metadata);
                if options.retain_modification_time {
                    set_file_mtime(to.as_ref(), mtime);
                }
                if options.retain_access_time {
                    set_file_atime(to.as_ref(), atime);
                }
            }
            Err(_) => {
                err!("Could not read metadata", ErrorKind::Other);
            }
        }
    }
    Ok(())
}
