use fs_extra::file::{self, CopyOptions};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "fs-extra-progress-safety-{}-{}",
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

fn check_zero_buffer_preserves_files(move_source: bool) {
    let directory = Directory::new();
    let source = directory.0.join("source");
    let destination = directory.0.join("destination");
    let source_contents = b"source must survive";
    let destination_contents = b"destination must survive";
    std::fs::write(&source, source_contents).unwrap();
    std::fs::write(&destination, destination_contents).unwrap();
    let callbacks = Cell::new(0);
    let copy = || {
        let options = CopyOptions::new().buffer_size(0).overwrite(true);
        let progress = |_| callbacks.set(callbacks.get() + 1);
        if move_source {
            file::move_file_with_progress(&source, &destination, &options, progress)
        } else {
            file::copy_with_progress(&source, &destination, &options, progress)
        }
    };

    let result = copy();
    assert_eq!(std::fs::read(&source).unwrap(), source_contents);
    assert_eq!(std::fs::read(&destination).unwrap(), destination_contents);
    assert!(result.is_err());
    assert_eq!(callbacks.get(), 0);

    std::fs::remove_file(&destination).unwrap();
    assert!(copy().is_err());
    assert!(!destination.exists());
    assert_eq!(std::fs::read(&source).unwrap(), source_contents);
    assert_eq!(callbacks.get(), 0);
}

#[test]
fn zero_buffer_progress_copy_preserves_source_and_destination() {
    check_zero_buffer_preserves_files(false);
}

#[test]
fn zero_buffer_progress_move_preserves_source_and_destination() {
    check_zero_buffer_preserves_files(true);
}

#[cfg(unix)]
fn check_permissions_before_progress(existing_destination: bool) {
    use std::os::unix::fs::PermissionsExt;

    let directory = Directory::new();
    let source = directory.0.join("source");
    let destination = directory.0.join("destination");
    let contents = b"private contents";
    std::fs::write(&source, contents).unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();
    if existing_destination {
        std::fs::write(&destination, b"old public contents").unwrap();
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    let callbacks = Cell::new(0);
    let options = CopyOptions::new().buffer_size(1).overwrite(true);
    let copied = file::copy_with_progress(&source, &destination, &options, |_| {
        callbacks.set(callbacks.get() + 1);
        assert_eq!(
            std::fs::metadata(&destination)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600,
            "source permissions must apply before any copied byte is reported"
        );
    })
    .unwrap();
    assert_eq!(copied, contents.len() as u64);
    assert_eq!(std::fs::read(&destination).unwrap(), contents);
    assert_eq!(
        std::fs::metadata(&destination)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(callbacks.get() > 0);
}

#[cfg(unix)]
#[test]
fn new_progress_destination_has_source_permissions_before_first_callback() {
    check_permissions_before_progress(false);
}

#[cfg(unix)]
#[test]
fn overwritten_progress_destination_has_source_permissions_before_first_callback() {
    check_permissions_before_progress(true);
}
