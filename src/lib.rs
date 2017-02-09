

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
/// ```rust,ignore
/// use std::path::Path;
/// use std::{thread, time};
/// use std::sync::mpsc::{self, TryRecvError};
///
/// extern crate fs_extra;
/// use fs_extra::file::*;
/// use fs_extra::error::*;
///
/// fn example_copy() -> Result<()> {
///     let path_from = Path::new("./temp");
///     let path_to = path_from.join("out");
///     let test_file = (path_from.join("test_file.txt"), path_to.join("test_file.txt"));
///
///
///     fs_extra::dir::create_all(&path_from, true)?;
///     fs_extra::dir::create_all(&path_to, true)?;
///
///     write_all(&test_file.0, "test_data")?;
///     assert!(test_file.0.exists());
///     assert!(!test_file.1.exists());
///
///
///     let mut options = CopyOptions::new();
///     options.buffer_size = 1;
///     let (tx, rx) = mpsc::channel();
///     thread::spawn(move || {
///         let handler = |process_info: TransitProcess| {
///             tx.send(process_info).unwrap();
///             thread::sleep(time::Duration::from_millis(500));
///         };
///         copy_with_progress(&test_file.0, &test_file.1, &options, handler).unwrap();
///         assert!(test_file.0.exists());
///         assert!(test_file.1.exists());
///
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
///
///
/// ```
pub mod file;

/// This module include extra methods for works with directories.
///
/// One of the distinguishing features is receipt information
/// about process work with directories and recursion operations.
///
/// # Example
/// ```rust,ignore
/// use std::path::Path;
/// use std::{thread, time};
/// use std::sync::mpsc::{self, TryRecvError};
///
/// extern crate fs_extra;
/// use fs_extra::dir::*;
/// use fs_extra::error::*;
///
/// fn example_copy() -> Result<()> {
///
///     let path_from = Path::new("./temp");
///     let path_to = path_from.join("out");
///     let test_folder = path_from.join("test_folder");
///     let dir = test_folder.join("dir");
///     let sub = dir.join("sub");
///     let file1 = dir.join("file1.txt");
///     let file2 = sub.join("file2.txt");
///
///     create_all(&sub, true)?;
///     create_all(&path_to, true)?;
///     fs_extra::file::write_all(&file1, "content1")?;
///     fs_extra::file::write_all(&file2, "content2")?;
///
///     assert!(dir.exists());
///     assert!(sub.exists());
///     assert!(file1.exists());
///     assert!(file2.exists());
///
///
///     let mut options = CopyOptions::new();
///     options.buffer_size = 1;
///     let (tx, rx) = mpsc::channel();
///     thread::spawn(move || {
///         let handler = |process_info: TransitProcess| {
///             tx.send(process_info).unwrap();
///             thread::sleep(time::Duration::from_millis(500));
///         };
///         copy_with_progress(&test_folder, &path_to, &options, handler).unwrap();
///     });
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
/// ```
///
pub mod dir;


use error::*;
use std::path::Path;


/// Copies list directories and files to another place using recursive method. This function will
/// also copy the permission bits of the original files to destionation files (not for
/// directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these case:
///
/// * List `from` contains  file or directory does not exist.
///
/// * List `from` contains  file or directory with invalid name.
///
/// * The current process does not have the permission rights to access to file from `lists from` or
/// `to`.
///
/// # Example
///
/// ```rust,ignore
///  extern crate fs_extra;
///  use fs_extra::dir::copy;
///
///  let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
///
///  // copy dir1 and file1.txt to target/dir1 and target/file1.txt
///  let mut from_paths = Vec::new();
///  from_paths.push("source/dir1");
///  from_paths.push("source/file.txt");
///  copy_items(&from_paths, "target", &options)?;
/// ```
///
pub fn copy_items<P, Q>(from: &Vec<P>, to: Q, options: &dir::CopyOptions) -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let mut result: u64 = 0;
    for item in from {
        let item = item.as_ref();
        if item.is_dir() {
            result += dir::copy(item, &to, options)?;
        } else {
            if let Some(file_name) = item.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    let mut file_options = file::CopyOptions::new();
                    file_options.overwrite = options.overwrite;
                    file_options.skip_exist = options.skip_exist;
                    result += file::copy(item, to.as_ref().join(file_name), &file_options)?;
                }
            } else {
                err!("Invalid file name", ErrorKind::InvalidFileName);
            }

        }
    }

    Ok(result)
}


/// A structure which include information about the current status of the copy or move directory.
pub struct TransitProcess {
    /// Copied bytes on this time for folder
    pub copied_bytes: u64,
    /// All the bytes which should to copy or move (dir size).
    pub total_bytes: u64,
    /// Copied bytes on this time for file.
    pub file_bytes_copied: u64,
    /// Size current copied file.
    pub file_total_bytes: u64,
    /// Name current copied file.
    pub file_name: String,
    /// Name current copied folder.
    pub dir_name: String,
}

impl Clone for TransitProcess {
    fn clone(&self) -> TransitProcess {
        TransitProcess {
            copied_bytes: self.copied_bytes,
            total_bytes: self.total_bytes,
            file_bytes_copied: self.file_bytes_copied,
            file_total_bytes: self.file_total_bytes,
            file_name: self.file_name.clone(),
            dir_name: self.dir_name.clone(),
        }
    }
}


/// Copies list directories and files to another place using recursive method, with recept
/// information about process. This function will also copy the permission bits of the
/// original files to destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these case:
///
/// * List `from` contains  file or directory does not exist.
///
/// * List `from` contains  file or directory with invalid name.
///
/// * The current process does not have the permission rights to access to file from `lists from` or
/// `to`.
///
/// # Example
/// ```rust,ignore
///
///  extern crate fs_extra;
///  use fs_extra::dir::copy;
///
///  let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
///  let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///  // copy dir1 and file1.txt to target/dir1 and target/file1.txt
///  let mut from_paths = Vec::new();
///  from_paths.push("source/dir1");
///  from_paths.push("source/file.txt");
///  copy_items_with_progress(&from_paths, "target", &options, handle)?;
/// ```
///
pub fn copy_items_with_progress<P, Q, F>(from: &Vec<P>,
                                         to: Q,
                                         options: &dir::CopyOptions,
                                         progress_handler: F)
                                         -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>,
          F: Fn(TransitProcess) -> ()
{

    let mut total_size = 0;
    let mut list_paths = Vec::new();
    for item in from {
        let item = item.as_ref();
        total_size += dir::get_size(item)?;
        list_paths.push(item);
    }

    let mut result: u64 = 0;
    let mut info_process = TransitProcess {
        copied_bytes: 0,
        total_bytes: total_size,
        file_bytes_copied: 0,
        file_total_bytes: 0,
        file_name: String::new(),
        dir_name: String::new(),
    };

    for item in list_paths {
        if item.is_dir() {
            if let Some(dir_name) = item.components().last() {
                if let Ok(dir_name) = dir_name.as_os_str().to_os_string().into_string() {
                    info_process.dir_name = dir_name;
                } else {
                    err!("Invalid folder from", ErrorKind::InvalidFolder);
                }
            } else {
                err!("Invalid folder from", ErrorKind::InvalidFolder);
            }

            let copied_bytes = result;
            let handler = |info: dir::TransitProcess| {
                info_process.copied_bytes = copied_bytes + info.copied_bytes;
                progress_handler(info_process.clone());
            };
            result += dir::copy_with_progress(item, &to, options, handler)?;
        } else {
            let mut file_options = file::CopyOptions::new();
            file_options.overwrite = options.overwrite;
            file_options.skip_exist = options.skip_exist;
            file_options.buffer_size = options.buffer_size;

            if let Some(file_name) = item.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    info_process.file_name = file_name.to_string();
                } else {
                    err!("Invalid file name", ErrorKind::InvalidFileName);
                }

            } else {
                err!("Invalid file name", ErrorKind::InvalidFileName);
            }

            info_process.file_bytes_copied = 0;
            info_process.file_total_bytes = item.metadata()?.len();

            let copied_bytes = result;
            let file_name = to.as_ref().join(info_process.file_name.clone());
            let handler = |info: file::TransitProcess| {
                info_process.copied_bytes = copied_bytes + info.copied_bytes;
                info_process.file_bytes_copied = info.copied_bytes;
                progress_handler(info_process.clone());
            };
            result += file::copy_with_progress(item, &file_name, &file_options, handler)?;
        }
    }

    Ok(result)
}

/// Moves list directories and files to another place using recursive method. This function will
/// also copy the permission bits of the original files to destionation files (not for
/// directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these case:
///
/// * List `from` contains  file or directory does not exist.
///
/// * List `from` contains  file or directory with invalid name.
///
/// * The current process does not have the permission rights to access to file from `lists from` or
/// `to`.
///
/// # Example
///
/// ```rust,ignore
///  extern crate fs_extra;
///  use fs_extra::dir::copy;
///
///  let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
///
///  // move dir1 and file1.txt to target/dir1 and target/file1.txt
///  let mut from_paths = Vec::new();
///  from_paths.push("source/dir1");
///  from_paths.push("source/file.txt");
///  move_items(&from_paths, "target", &options)?;
/// ```
///
pub fn move_items<P, Q>(from_items: &Vec<P>, to: Q, options: &dir::CopyOptions) -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let mut total_size = 0;
    let mut list_paths = Vec::new();
    for item in from_items {
        let item = item.as_ref();
        total_size += dir::get_size(item)?;
        list_paths.push(item);
    }

    let mut result = 0;
    let mut info_process = TransitProcess {
        copied_bytes: 0,
        total_bytes: total_size,
        file_bytes_copied: 0,
        file_total_bytes: 0,
        file_name: String::new(),
        dir_name: String::new(),
    };

    for item in list_paths {
        if item.is_dir() {
            if let Some(dir_name) = item.components().last() {
                if let Ok(dir_name) = dir_name.as_os_str().to_os_string().into_string() {
                    info_process.dir_name = dir_name;
                } else {
                    err!("Invalid folder from", ErrorKind::InvalidFolder);
                }
            } else {
                err!("Invalid folder from", ErrorKind::InvalidFolder);
            }

            result += dir::move_dir(item, &to, options)?;
        } else {
            let mut file_options = file::CopyOptions::new();
            file_options.overwrite = options.overwrite;
            file_options.skip_exist = options.skip_exist;
            file_options.buffer_size = options.buffer_size;

            if let Some(file_name) = item.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    info_process.file_name = file_name.to_string();
                } else {
                    err!("Invalid file name", ErrorKind::InvalidFileName);
                }

            } else {
                err!("Invalid file name", ErrorKind::InvalidFileName);
            }

            info_process.file_bytes_copied = 0;
            info_process.file_total_bytes = item.metadata()?.len();

            let file_name = to.as_ref().join(info_process.file_name.clone());
            result += file::move_file(item, &file_name, &file_options)?;
        }
    }

    Ok(result)

}


/// Moves list directories and files to another place using recursive method, with recept
/// information about process. This function will also copy the permission bits of the
/// original files to destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these case:
///
/// * List `from` contains  file or directory does not exist.
///
/// * List `from` contains  file or directory with invalid name.
///
/// * The current process does not have the permission rights to access to file from `lists from` or
/// `to`.
///
/// # Example
///
/// ```rust,ignore
///  extern crate fs_extra;
///  use fs_extra::dir::copy;
///
///  let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
///  let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///  // move dir1 and file1.txt to target/dir1 and target/file1.txt
///  let mut from_paths = Vec::new();
///  from_paths.push("source/dir1");
///  from_paths.push("source/file.txt");
///  move_items_with_progress(&from_paths, "target", &options, handle)?;
/// ```
///
pub fn move_items_with_progress<P, Q, F>(from_items: &Vec<P>,
                                         to: Q,
                                         options: &dir::CopyOptions,
                                         progress_handler: F)
                                         -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>,
          F: Fn(TransitProcess) -> ()
{
    let mut total_size = 0;
    let mut list_paths = Vec::new();
    for item in from_items {
        let item = item.as_ref();
        total_size += dir::get_size(item)?;
        list_paths.push(item);
    }

    let mut result = 0;
    let mut info_process = TransitProcess {
        copied_bytes: 0,
        total_bytes: total_size,
        file_bytes_copied: 0,
        file_total_bytes: 0,
        file_name: String::new(),
        dir_name: String::new(),
    };

    for item in list_paths {
        if item.is_dir() {
            if let Some(dir_name) = item.components().last() {
                if let Ok(dir_name) = dir_name.as_os_str().to_os_string().into_string() {
                    info_process.dir_name = dir_name;
                } else {
                    err!("Invalid folder from", ErrorKind::InvalidFolder);
                }
            } else {
                err!("Invalid folder from", ErrorKind::InvalidFolder);
            }

            let copied_bytes = result;
            let handler = |info: dir::TransitProcess| {
                info_process.copied_bytes = copied_bytes + info.copied_bytes;
                progress_handler(info_process.clone());
            };
            result += dir::move_dir_with_progress(item, &to, options, handler)?;

        } else {
            let mut file_options = file::CopyOptions::new();
            file_options.overwrite = options.overwrite;
            file_options.skip_exist = options.skip_exist;
            file_options.buffer_size = options.buffer_size;

            if let Some(file_name) = item.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    info_process.file_name = file_name.to_string();
                } else {
                    err!("Invalid file name", ErrorKind::InvalidFileName);
                }

            } else {
                err!("Invalid file name", ErrorKind::InvalidFileName);
            }

            info_process.file_bytes_copied = 0;
            info_process.file_total_bytes = item.metadata()?.len();

            let copied_bytes = result;
            let file_name = to.as_ref().join(info_process.file_name.clone());
            let handler = |info: file::TransitProcess| {
                info_process.copied_bytes = copied_bytes + info.copied_bytes;
                info_process.file_bytes_copied = info.copied_bytes;
                progress_handler(info_process.clone());
            };
            result += file::move_file_with_progress(item, &file_name, &file_options, handler)?;
        }

    }
    Ok(result)

}

/// Removes list files or directories.
///
/// # Example
///
/// ```rust,ignore
///  let mut from_paths = Vec::new();
///  from_paths.push("source/dir1");
///  from_paths.push("source/file.txt");
///
///  remove_items(&from_paths).unwrap();
/// ```
///
pub fn remove_items<P>(from_items: &Vec<P>) -> Result<()>
    where P: AsRef<Path>
{
    for item in from_items {
        let item = item.as_ref();
        if item.is_dir() {
            dir::remove(item)?;
        } else {
            file::remove(item)?
        }
    }

    Ok(())
}
