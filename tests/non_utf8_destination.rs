// macOS filesystems reject non-UTF8 filenames before either copy API can be exercised.
#![cfg(all(unix, not(target_os = "macos")))]

use fs_extra::error::ErrorKind;
use fs_extra::file::{self, CopyOptions};
use std::cell::Cell;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        let parent = std::env::temp_dir();
        loop {
            let path = parent.join(format!(
                "fs-extra-non-utf8-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create test directory: {}", error),
            }
        }
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn check_existing_non_utf8_destination(with_progress: bool) {
    let directory = Directory::new();
    let source = directory.0.join("source");
    let destination = directory
        .0
        .join(OsString::from_vec(b"destination-\xff".to_vec()));
    let original = b"do not overwrite";
    let replacement = b"replacement contents";
    std::fs::write(&source, replacement).unwrap();
    std::fs::write(&destination, original).unwrap();
    assert!(destination.to_str().is_none());

    let callbacks = Cell::new(0);
    let copy = |options: &CopyOptions| {
        if with_progress {
            file::copy_with_progress(&source, &destination, options, |_| {
                callbacks.set(callbacks.get() + 1)
            })
        } else {
            file::copy(&source, &destination, options)
        }
    };

    let result = copy(&CopyOptions::new());
    assert_eq!(std::fs::read(&destination).unwrap(), original);
    assert!(matches!(result.unwrap_err().kind, ErrorKind::AlreadyExists));
    assert_eq!(callbacks.get(), 0);

    assert_eq!(copy(&CopyOptions::new().skip_exist(true)).unwrap(), 0);
    assert_eq!(std::fs::read(&destination).unwrap(), original);
    assert_eq!(callbacks.get(), 0);

    assert_eq!(
        copy(&CopyOptions::new().overwrite(true)).unwrap(),
        replacement.len() as u64
    );
    assert_eq!(std::fs::read(&destination).unwrap(), replacement);
    if with_progress {
        assert!(callbacks.get() > 0);
    }
}

#[test]
fn copy_preserves_an_existing_non_utf8_destination_unless_overwrite_is_enabled() {
    check_existing_non_utf8_destination(false);
}

#[test]
fn progress_copy_preserves_an_existing_non_utf8_destination_unless_overwrite_is_enabled() {
    check_existing_non_utf8_destination(true);
}
