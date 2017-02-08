use std::path::{Path, PathBuf};
use error::*;
use std::fs::{create_dir, create_dir_all, remove_dir_all, read_dir};

///	Options and flags which can be used to configure how a file will be  copied  or moved.
pub struct CopyOptions {
    /// Sets the option true for overwrite existing files.
    pub overwrite: bool,
    /// Sets the option true for skipe existing files.
    pub skip_exist: bool,
    /// Sets buffer size for copy/move work only with receipt information about process work.
    pub buffer_size: usize,
}

impl CopyOptions {
    /// Initialize struct CopyOptions with default value.
    ///
    /// ```rust,ignore
    /// overwrite: false
    ///
    /// skip_exist: false
    ///
    /// buffer_size: 64000 //64kb
    ///```
    pub fn new() -> CopyOptions {
        CopyOptions {
            overwrite: false,
            skip_exist: false,
            buffer_size: 64000, //64kb
        }
    }
}

/// A structure which imclude information about directory
struct DirContent {
    /// Directory size.
    dir_size: u64,
    /// List all files directory and sub directories.
    files: Vec<String>,
    /// List all folders and sub folders directory.
    directories: Vec<String>,
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
}

impl Clone for TransitProcess {
    fn clone(&self) -> TransitProcess {
        TransitProcess {
            copied_bytes: self.copied_bytes,
            total_bytes: self.total_bytes,
            file_bytes_copied: self.file_bytes_copied,
            file_total_bytes: self.file_total_bytes,
            file_name: self.file_name.clone(),
        }
    }
}

/// Creates a new, empty directory at the provided path.
///
/// This function takes to arguments:
///
/// * `path` - Path to new directory.
///
/// * `erase` - If set true and folder exist, then folder will be erased.
///
/// #Errors
///
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
///
/// * User lacks permissions to create directory at `path`.
///
/// * `path` already exists if `erase` set false.
///
/// #Examples
///
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::dir::create;
///
/// create("dir", false); // create directory
/// ```
pub fn create<P>(path: P, erase: bool) -> Result<()>
    where P: AsRef<Path>
{
    if erase && path.as_ref().exists() {
        remove(&path)?;
    }
    Ok(create_dir(&path)?)
}

/// Recursively create a directory and all of its parent components if they are missing.
///
/// This function takes to arguments:
///
/// * `path` - Path to new directory.
///
/// * `erase` - If set true and folder exist, then folder will be erased.
///
///#Errors
///
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
///
/// * User lacks permissions to create directory at `path`.
///
/// * `path` already exists if `erase` set false.
///
/// #Examples
///
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::dir::create_all;
///
/// create_all("/some/dir", false); // create directory some and dir
pub fn create_all<P>(path: P, erase: bool) -> Result<()>
    where P: AsRef<Path>
{
    if erase && path.as_ref().exists() {
        remove(&path)?;
    }
    Ok(create_dir_all(&path)?)
}

/// Copies the directory contents from one place to another using recursive method.
/// This function will also copy the permission bits of the original files to
/// destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a directory.
/// * This `from` directory does not exist.
/// * Invalid folder name for `from` or `to`.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
///
///     extern crate fs_extra;
///     use fs_extra::dir::copy;
///
///     let options = CopyOptions::new(); //Initialize default values for CopyOptions
///
///     // copy source/dir1 to target/dir1
///     copy("source/dir1", "target/dir1", &options)?;
///
/// ```
pub fn copy<P, Q>(from: P, to: Q, options: &CopyOptions) -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let from = from.as_ref();

    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist", msg);
            err!(&msg, ErrorKind::NotFound);
        }
        err!("Path does not exist", ErrorKind::NotFound);
    }

    let mut to: PathBuf = to.as_ref().to_path_buf();
    if !from.is_dir() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a directory!", msg);
            err!(&msg, ErrorKind::InvalidFolder);
        }
        err!("Path is not a directory!", ErrorKind::InvalidFolder);
    }

    if let Some(dir_name) = from.components().last() {
        to.push(dir_name.as_os_str());
    } else {
        err!("Invalid folder from", ErrorKind::InvalidFolder);
    }

    if !to.exists() {
        create(&to, false)?;
    }

    let mut result: u64 = 0;
    for entry in read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result += copy(path, to.clone(), &options)?;
        } else {
            let mut to = to.to_path_buf();
            match path.file_name() {
                None => err!("No file name"),
                Some(file_name) => {
                    to.push(file_name);

                    let mut file_options = super::file::CopyOptions::new();
                    file_options.overwrite = options.overwrite;
                    file_options.skip_exist = options.skip_exist;
                    result += super::file::copy(&path, to.as_path().clone(), &file_options)?;

                }
            }
        }
    }

    Ok(result)
}


fn get_dir_content<P>(path: P) -> Result<DirContent>
    where P: AsRef<Path>
{
    let mut directories = Vec::new();
    let mut files = Vec::new();
    let mut dir_size = 0;
    let item = path.as_ref().to_str();
    if !item.is_some() {
        err!("Invalid path", ErrorKind::InvalidPath);
    }
    let item = item.unwrap().to_string();

    if path.as_ref().is_dir() {
        directories.push(item);
    } else {
        dir_size = Path::new(&item).metadata()?.len();
        files.push(item);
    }
    if path.as_ref().is_dir() {
        for entry in read_dir(&path)? {
            let _path = entry?.path();

            match get_dir_content(_path) {
                Ok(items) => {
                    let mut _files = items.files;
                    let mut _dirrectories = items.directories;
                    dir_size += items.dir_size;
                    files.append(&mut _files);
                    directories.append(&mut _dirrectories);
                }
                Err(err) => return Err(err),
            }
        }
    }
    Ok(DirContent {
        dir_size: dir_size,
        files: files,
        directories: directories,
    })
}

/// Copies the directory contents from one place to another using recursive method,
/// with recept information about process. This function will also copy the
/// permission bits of the original files to destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a directory.
/// * This `from` directory does not exist.
/// * Invalid folder name for `from` or `to`.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
///     extern crate fs_extra;
///     use fs_extra::dir::copy;
///
///     let options = CopyOptions::new(); //Initialize default values for CopyOptions
///     let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///
///     // copy source/dir1 to target/dir1
///     copy_with_progress("source/dir1", "target/dir1", &options, handle)?;
///
/// ```
pub fn copy_with_progress<P, Q, F>(from: P,
                                   to: Q,
                                   options: &CopyOptions,
                                   progress_handler: F)
                                   -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>,
          F: Fn(TransitProcess) -> ()
{

    let from = from.as_ref();

    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist", msg);
            err!(&msg, ErrorKind::NotFound);
        }
        err!("Path does not exist", ErrorKind::NotFound);
    }

    let mut to: PathBuf = to.as_ref().to_path_buf();
    if !from.is_dir() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a directory!", msg);
            err!(&msg, ErrorKind::InvalidFolder);
        }
        err!("Path is not a directory!", ErrorKind::InvalidFolder);
    }

    if let Some(dir_name) = from.components().last() {
        to.push(dir_name.as_os_str());
    } else {
        err!("Invalid folder from", ErrorKind::InvalidFolder);
    }

    let dir_content = get_dir_content(from)?;
    for directory in dir_content.directories {
        let tmp_to = Path::new(&directory).strip_prefix(from)?;
        let dir = to.join(&tmp_to);
        if !dir.exists() {
            create(dir, false)?;
        }

    }

    let mut result: u64 = 0;
    let mut info_process = TransitProcess {
        copied_bytes: 0,
        total_bytes: dir_content.dir_size,
        file_bytes_copied: 0,
        file_total_bytes: 0,
        file_name: String::new(),
    };

    for file in dir_content.files {
        let mut to = to.to_path_buf();
        let tp = Path::new(&file).strip_prefix(from)?;
        let path = to.join(&tp);

        let file_name = path.file_name();
        if !file_name.is_some() {
            err!("No file name");
        }
        let file_name = file_name.unwrap();
        to.push(file_name);

        let file_options = super::file::CopyOptions {
            overwrite: options.overwrite,
            skip_exist: options.skip_exist,
            buffer_size: options.buffer_size,
        };

        if let Some(file_name) = file_name.to_str() {
            info_process.file_name = file_name.to_string();
        } else {
            err!("Invalid file name", ErrorKind::InvalidFileName);
        }

        info_process.file_bytes_copied = 0;
        info_process.file_total_bytes = Path::new(&file).metadata()?.len();

        let copied_bytes = result;
        let _progress_hadler = |info: super::file::TransitProcess| {
            info_process.copied_bytes = copied_bytes + info.copied_bytes;
            info_process.file_bytes_copied = info.copied_bytes;
            progress_handler(info_process.clone());

        };

        result += super::file::copy_with_progress(&file, &path, &file_options, _progress_hadler)?;

    }

    Ok(result)
}


/// Moves the directory contents from one place to another.
/// This function will also copy the permission bits of the original files to
/// destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a directory.
/// * This `from` directory does not exist.
/// * Invalid folder name for `from` or `to`.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
///
///     extern crate fs_extra;
///     use fs_extra::dir::move_dir;
///
///     let options = CopyOptions::new(); //Initialize default values for CopyOptions
///
///     // move source/dir1 to target/dir1
///     move_dir("source/dir1", "target/dir1", &options)?;
///
/// ```
pub fn move_dir<P, Q>(from: P, to: Q, options: &CopyOptions) -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let mut is_remove = true;
    if options.skip_exist && to.as_ref().exists() && !options.overwrite {
        is_remove = false;
    }
    let result = copy(&from, to, options)?;
    if is_remove {
        remove(from)?;
    }

    Ok(result)

}

/// Moves the directory contents from one place to another with recept information about process.
/// This function will also copy the permission bits of the original files to
/// destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a directory.
/// * This `from` directory does not exist.
/// * Invalid folder name for `from` or `to`.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
///
///     extern crate fs_extra;
///     use fs_extra::dir::move_dir_with_progress;
///
///     let options = CopyOptions::new(); //Initialize default values for CopyOptions
///     let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///
///     // move source/dir1 to target/dir1
///     move_dir_with_progress("source/dir1", "target/dir1", &options, handle)?;
///
/// ```
pub fn move_dir_with_progress<P, Q, F>(from: P,
                                       to: Q,
                                       options: &CopyOptions,
                                       progress_handler: F)
                                       -> Result<u64>
    where P: AsRef<Path>,
          Q: AsRef<Path>,
          F: Fn(TransitProcess) -> ()
{
    let mut is_remove = true;
    if options.skip_exist && to.as_ref().exists() && !options.overwrite {
        is_remove = false;
    }
    let result = copy_with_progress(&from, to, options, progress_handler)?;
    if is_remove {
        remove(from)?;
    }

    Ok(result)
}


/// Removes directory.
///
/// # Example
/// ```rust,ignore
///
///     extern crate fs_extra;
///     use fs_extra::dir::remove;
///
///     remove("source/dir1"); // remove dir1
/// ```
pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
    if path.as_ref().exists() {
        Ok(remove_dir_all(path)?)
    } else {
        Ok(())
    }
}
