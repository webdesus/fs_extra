

macro_rules! err {
    ($text:expr, $kind:expr) => {
        return Err(Error::new($kind, $text));
    };

    ($text:expr) => {
        err!($text, ErrorKind::Other)
    };
}

/// The error type for fs_extra operations with files and folder.
pub mod error;
/// This module include extra methods for works with files.
///
/// One of the distinguishing features is receipt information
/// about process work with files.
///
/// # Example
/// ```
/// use std::path::{Path, PathBuf};
/// use std::{thread, time};
/// use std::sync::mpsc::{self, TryRecvError};
///
/// extern crate fs_extra;
/// use fs_extra::file::*;
/// use fs_extra::error::*;
///
/// fn example_copy() -> Result<()> {
///     let mut test_file = PathBuf::from("./temp");
///     test_file.push("temp_folder");
///     let mut test_file_out = test_file.clone();
///     fs_extra::dir::create_all(&test_file, true)?;
///     test_file.push("test.txt");
///     test_file_out.push("out");
///     fs_extra::dir::create_all(&test_file_out, true)?;
///     test_file_out.push("test.txt");
///
///     write_all(&test_file, "test_data")?;
///     assert!(test_file.exists());
///     assert!(!test_file_out.exists());
///     let mut options = CopyOptions::new();
///     options.buffer_size = 1;
///     let (tx, rx) = mpsc::channel();
///     thread::spawn(move || {
///         let handler = |process_info: TransitProcess| {
///             tx.send(process_info).unwrap();
///             thread::sleep(time::Duration::from_millis(500));
///         };
///         copy_with_progress(&test_file, &test_file_out, &options, handler).unwrap();
///         assert!(test_file.exists());
///         assert!(test_file_out.exists());
///     });
///     loop {
///         match rx.try_recv() {
///             Ok(process_info) => {
///                 println!("{} of {} bytes",
///                          process_info.copied_bytes,
///                          process_info.total_bytes);
///             }
///             Err(TryRecvError::Disconnected) => {
///                 println!("finished");
///                 break;
///             }
///             Err(TryRecvError::Empty) => {}
///         }
///     }
///     Ok(())
///
/// }
///
///
/// fn main() {
///     example_copy();
/// }
///
/// ```
pub mod file;

/// This module include extra methods for works with directories.
///
/// One of the distinguishing features is receipt information
/// about process work with directories and recursion operations.
///
/// # Example
/// ```
/// use std::path::{Path, PathBuf};
/// use std::{thread, time};
/// use std::sync::mpsc::{self, TryRecvError};
///
/// extern crate fs_extra;
/// use fs_extra::dir::*;
/// use fs_extra::error::*;
///
/// fn example_copy() -> Result<()> {
///
///     let mut path_from = PathBuf::from("./temp");
///     let test_name = "dir";
///     path_from.push("test_folder");
///     let mut path_to = path_from.clone();
///     path_to.push("out");
///     path_from.push(&test_name);
///
///     create_all(&path_from, true)?;
///     assert!(path_from.exists());
///     create_all(&path_to, true)?;
///     assert!(path_to.exists());
///
///     let mut file1_path = path_from.clone();
///     file1_path.push("test1.txt");
///     let content1 = "content";
///     fs_extra::file::write_all(&file1_path, &content1)?;
///     assert!(file1_path.exists());
///
///     let mut sub_dir_path = path_from.clone();
///     sub_dir_path.push("sub");
///     create(&sub_dir_path, true)?;
///     let mut file2_path = sub_dir_path.clone();
///     file2_path.push("test2.txt");
///     let content2 = "content2";
///     fs_extra::file::write_all(&file2_path, &content2)?;
///     assert!(file2_path.exists());
///
///     let mut options = CopyOptions::new();
///
///     options.buffer_size = 1;
///     let (tx, rx) = mpsc::channel();
///     thread::spawn(move || {
///         let handler = |process_info: TransitProcess| {
///             tx.send(process_info).unwrap();
///             thread::sleep(time::Duration::from_millis(500));
///         };
///         copy_with_progress(&path_from, &path_to, &options, handler).unwrap();
///     });
///
///
///     loop {
///         match rx.try_recv() {
///             Ok(process_info) => {
///                 println!("{} of {} bytes",
///                          process_info.copied_bytes,
///                          process_info.total_bytes);
///             }
///             Err(TryRecvError::Disconnected) => {
///                 println!("finished");
///                 break;
///             }
///             Err(TryRecvError::Empty) => {}
///         }
///     }
///     Ok(())
///
/// }
/// fn main() {
///     example_copy();
/// }
pub mod dir;
