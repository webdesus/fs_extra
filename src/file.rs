use error::{Error, ErrorKind, Result};
use std;
use std::cmp;
use std::fs::{remove_file, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

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
    ///
    /// overwrite: false
    ///
    /// skip_exist: false
    ///
    /// buffer_size: 64000 //64kb
    /// ```
    pub fn new() -> CopyOptions {
        CopyOptions {
            overwrite: false,
            skip_exist: false,
            buffer_size: 64000, //64kb
        }
    }
}
/// A structure  which include information about the current status of the copy or move file.
pub struct TransitProcess {
    /// Copied bytes on this time.
    pub copied_bytes: u64,
    /// All the bytes which should to copy or move.
    pub total_bytes: u64,
}

/// Copies the contents of one file to another. This function will also copy the permission
/// bits of the original file to the destination file.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a file.
/// * This `from` file does not exist.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
///
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::copy;
///
/// let options = CopyOptions::new(); //Initialize default values for CopyOptions
/// copy("dir1/foo.txt", "dir2/bar.txt", &options)?; // Copy dir1/foo.txt to dir2/bar.txt
///
/// ```
pub fn copy<P, Q>(from: P, to: Q, options: &CopyOptions) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist or you don't have access!", msg);
            err!(&msg, ErrorKind::NotFound);
        }
        err!(
            "Path does not exist or you don't have access!",
            ErrorKind::NotFound
        );
    }

    if !from.is_file() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a file!", msg);
            err!(&msg, ErrorKind::InvalidFile);
        }
        err!("Path is not a file!", ErrorKind::InvalidFile);
    }

    if !options.overwrite && to.as_ref().exists() {
        if options.skip_exist {
            return Ok(0);
        }

        if let Some(msg) = to.as_ref().to_str() {
            let msg = format!("Path \"{}\" is exist", msg);
            err!(&msg, ErrorKind::AlreadyExists);
        }
    }

    Ok(std::fs::copy(from, to)?)
}

/// Copies the contents of one file to another with recept information about process.
/// This function will also copy the permission bits of the original file to the
/// destination file.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a file.
/// * This `from` file does not exist.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::copy_with_progress;
///
/// let options = CopyOptions::new(); //Initialize default values for CopyOptions
/// let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///
/// // Copy dir1/foo.txt to dir2/foo.txt
/// copy_with_progress("dir1/foo.txt", "dir2/foo.txt", &options, handle)?;
///
/// ```
pub fn copy_with_progress<P, Q, F>(
    from: P,
    to: Q,
    options: &CopyOptions,
    mut progress_handler: F,
) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: FnMut(TransitProcess) -> (),
{
    let from = from.as_ref();
    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist or you don't have access!", msg);
            err!(&msg, ErrorKind::NotFound);
        }
        err!(
            "Path does not exist or you don't have access!",
            ErrorKind::NotFound
        );
    }

    if !from.is_file() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a file!", msg);
            err!(&msg, ErrorKind::InvalidFile);
        }
        err!("Path is not a file!", ErrorKind::InvalidFile);
    }

    if !options.overwrite && to.as_ref().exists() {
        if options.skip_exist {
            return Ok(0);
        }

        if let Some(msg) = to.as_ref().to_str() {
            let msg = format!("Path \"{}\" is exist", msg);
            err!(&msg, ErrorKind::AlreadyExists);
        }
    }
    let mut file_from = File::open(from)?;
    let mut buf = vec![0; options.buffer_size];
    let file_size = file_from.metadata()?.len();
    let mut copied_bytes: u64 = 0;

    let mut file_to = File::create(to)?;
    while !buf.is_empty() {
        match file_from.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                file_to.write(&mut buf[..n])?;
                copied_bytes = copied_bytes + n as u64;
                let data = TransitProcess {
                    copied_bytes: copied_bytes,
                    total_bytes: file_size,
                };
                progress_handler(data);
            }
            Err(ref e) if e.kind() == ::std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(::std::convert::From::from(e)),
        }
    }
    Ok(file_size)
}

/// Moves file from one place to another. This function will also copy the permission
/// bits of the original file to the destination file.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a file.
/// * This `from` file does not exist.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::move_file;
///
/// let options = CopyOptions::new(); //Initialize default values for CopyOptions
/// move_file("dir1/foo.txt", "dir2/foo.txt", &options)?; // Move dir1/foo.txt to dir2/foo.txt
///
/// ```
pub fn move_file<P, Q>(from: P, to: Q, options: &CopyOptions) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
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

/// Moves file from one place to another with recept information about process.
/// This function will also copy the permission bits of the original file to the
/// destination file.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a file.
/// * This `from` file does not exist.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::move_file;
///
/// let options = CopyOptions::new(); //Initialize default values for CopyOptions
/// let handle = |process_info: TransitProcess|  println!("{}", process_info.total_bytes);
///
/// // Move dir1/foo.txt to dir2/foo.txt
/// move_file("dir1/foo.txt", "dir2/foo.txt", &options, handle)?;
///
/// ```
pub fn move_file_with_progress<P, Q, F>(
    from: P,
    to: Q,
    options: &CopyOptions,
    progress_handler: F,
) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: FnMut(TransitProcess) -> (),
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

/// Removes a file from the filesystem.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * The current process does not have the permission rights to access `path`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::remove;
///
/// remove("foo.txt" )?; // Remove foo.txt
///
/// ```
pub fn remove<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if path.as_ref().exists() {
        Ok(remove_file(path)?)
    } else {
        Ok(())
    }
}

/// Read file content, placing him into `String`.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `path` is not a file.
/// * This `path` file does not exist.
/// * The current process does not have the permission rights to access `path`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::read_to_string;
///
/// let file_content = read_to_string("foo.txt" )?; // Get file conent from foo.txt
/// println!("{}", file_content);
///
/// ```
pub fn read_to_string<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.exists() && !path.is_file() {
        if let Some(msg) = path.to_str() {
            let msg = format!("Path \"{}\" is not a file!", msg);
            err!(&msg, ErrorKind::InvalidFile);
        }
        err!("Path is not a file!", ErrorKind::InvalidFile);
    }

    let mut file = File::open(path)?;
    let mut result = String::new();
    file.read_to_string(&mut result)?;

    Ok(result)
}

/// Write `String` content into inside target file.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `path` is not a file.
/// * This `path` file does not exist.
/// * The current process does not have the permission rights to access `path`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::file::read_to_string;
///
/// write_all("foo.txt", "conents" )?; // Create file foo.txt and send content inside
///
/// ```
pub fn write_all<P>(path: P, content: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.exists() && !path.is_file() {
        if let Some(msg) = path.to_str() {
            let msg = format!("Path \"{}\" is not a file!", msg);
            err!(&msg, ErrorKind::InvalidFile);
        }
        err!("Path is not a file!", ErrorKind::InvalidFile);
    }

    let mut f = File::create(path)?;

    Ok(f.write_all(content.as_bytes())?)
}

/// Information block which should be changed
pub struct Block {
    pub begin: u64,
    pub len: u64,
    pub data: Vec<u8>,
}
pub trait ExtraFile {
    fn change_block(&mut self, block: &Block, buffer_size: u64);
    fn change_blocks(&mut self, block: Vec<Block>, buffer_size: u64);
}

impl ExtraFile for File {
    fn change_block(&mut self, block: &Block, buffer_size: u64) {
        let file_len = self.metadata().unwrap().len();
        if file_len < (block.begin + block.len) {
            //TODO make as error
            panic!("The selected block is out of bounds file size!");
        }

        let mut bytes_to_move = file_len - (block.begin + block.len);
        let mut move_cursor = file_len;
        let data_len = block.data.len() as u64;
        let is_move_left = block.len > data_len;
        if is_move_left {
            move_cursor = block.begin;
        }
        if block.len == data_len {
            bytes_to_move = 0;
        }
        while bytes_to_move != 0 {
            let bytes_this_time: u64 = cmp::min(buffer_size, bytes_to_move);
            let mut r_buffer = vec![0; bytes_this_time as usize];
            let rd_off: u64;
            let wr_off: u64;
            if is_move_left {
                rd_off = move_cursor + block.len;
                wr_off = rd_off - block.len + data_len;
                move_cursor += bytes_this_time;
            } else {
                rd_off = move_cursor - bytes_this_time;
                wr_off = rd_off - block.len + data_len;
                move_cursor -= bytes_this_time;
            }
            self.seek(SeekFrom::Start(rd_off)).unwrap();
            self.read(&mut r_buffer[..]).unwrap();
            self.seek(SeekFrom::Start(wr_off)).unwrap();
            self.write_all(&r_buffer).unwrap();
            bytes_to_move -= bytes_this_time;
        }
        self.seek(SeekFrom::Start(block.begin)).unwrap();
        self.write_all(&block.data).unwrap();
        self.flush().unwrap();
        if is_move_left {
            self.set_len(file_len - (block.len - data_len)).unwrap();
        }
    }
    fn change_blocks(&mut self, blocks: Vec<Block>, buffer_size: u64) {
        let mut blocks = blocks;
        blocks.sort_by(|a, b| b.begin.cmp(&a.begin));
        for b in blocks {
            self.change_block(&b, buffer_size.clone());
        }
    }
}
