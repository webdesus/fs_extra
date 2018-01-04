# fs_extra

A Rust library for more work functionality with file system.

[![Build Status](https://travis-ci.org/webdesus/fs_extra.svg)](https://travis-ci.org/webdesus/fs_extra)
[![Crates.io Status](https://img.shields.io/crates/v/fs_extra.svg)](https://crates.io/crates/fs_extra)
[![Docs](https://docs.rs/fs_extra/badge.svg)](https://docs.rs/fs_extra)

[Documentation](https://docs.rs/fs_extra)

[Migrations to 1.x.x version](https://github.com/webdesus/fs_extra/wiki/Migrations-to-1.x.x-version)


## Key features:

* Copy files (optionally with information about the progress).

* Copy directories recursively (optionally with information about the progress).

* Move files (optionally with information about the progress).

* Move directories recursively (optionally with information about the progress).

* One method for create and write `String` content in file.

* One method for open and read `String` content from file.

* Get size folder

* Get collection directory entries 

## Functions:

| Function | Description |
| ------------- | ------------- |
| [fs_extra::copy_items](https://docs.rs/fs_extra/*/fs_extra/fn.copy_items.html)  | Copies list directories and files to another place using recursive method |
| [fs_extra::copy_items_with_progress](https://docs.rs/fs_extra/*/fs_extra/fn.copy_items_with_progress.html)  | Copies list directories and files to another place using recursive method, with recept information about process |
| [fs_extra::move_items](https://docs.rs/fs_extra/*/fs_extra/fn.move_items.html)  | Moves list directories and files to another place using recursive method |
| [fs_extra::move_items_with_progress](https://docs.rs/fs_extra/*/fs_extra/fn.move_items_with_progress.html)  | Moves list directories and files to another place using recursive method, with recept information about process |
| [fs_extra::remove_items](https://docs.rs/fs_extra/*/fs_extra/fn.remove_items.html)  | Removes list files or directories |
| [fs_extra::file::copy](https://docs.rs/fs_extra/*/fs_extra/file/fn.copy.html)  | Copies the contents of one file to another |
| [fs_extra::file::copy_with_progress](https://docs.rs/fs_extra/*/fs_extra/file/fn.copy_with_progress.html)  | Copies the contents of one file to another with recept information about process  |
| [fs_extra::file::move_file](https://docs.rs/fs_extra/*/fs_extra/file/fn.move_file.html)  | Moves file from one place to another  |
| [fs_extra::file::move_file_with_progress](https://docs.rs/fs_extra/*/fs_extra/file/fn.move_file_with_progress.html)  | Moves file from one place to another with recept information about process  |
| [fs_extra::file::remove](https://docs.rs/fs_extra/*/fs_extra/file/fn.remove.html)  | Removes a file from the filesystem  |
| [fs_extra::file::read_to_string](https://docs.rs/fs_extra/*/fs_extra/file/fn.read_to_string.html)  | Read file content, placing him into `String`  |
| [fs_extra::file::write_all](https://docs.rs/fs_extra/*/fs_extra/file/fn.write_all.html)  | Write `String` content into inside target file  |
| [fs_extra::dir::create](https://docs.rs/fs_extra/*/fs_extra/dir/fn.create.html)  | Creates a new, empty directory at the provided path  |
| [fs_extra::dir::create_all](https://docs.rs/fs_extra/*/fs_extra/dir/fn.create_all.html)  | Recursively create a directory and all of its parent components if they are missing  |
| [fs_extra::dir::copy](https://docs.rs/fs_extra/*/fs_extra/dir/fn.copy.html)  | Copies the directory contents from one place to another using recursive method  |
| [fs_extra::dir::copy_with_progress](https://docs.rs/fs_extra/*/fs_extra/dir/fn.copy_with_progress.html)  | Copies the directory contents from one place to another using recursive method, with recept information about process]()  |
| [fs_extra::dir::move_dir](https://docs.rs/fs_extra/*/fs_extra/dir/fn.move_dir.html)  | Moves the directory contents from one place to another  |
| [fs_extra::dir::move_dir_with_progress](https://docs.rs/fs_extra/*/fs_extra/dir/fn.move_dir_with_progress.html)  | Moves the directory contents from one place to another with recept information about process  |
| [fs_extra::dir::remove](https://docs.rs/fs_extra/*/fs_extra/dir/fn.remove.html)  | Removes directory  |
| [fs_extra::dir::get_size](https://docs.rs/fs_extra/*/fs_extra/dir/fn.get_size.html)  | Returns the size of the file or directory  |
| [fs_extra::dir::get_dir_content](https://docs.rs/fs_extra/*/fs_extra/dir/fn.get_dir_content.html)  | Return DirContent which containt information about directory  |
| [fs_extra::dir::get_details_entry](https://docs.rs/fs_extra/*/fs_extra/dir/fn.get_details_entry.html)  | Returned information about directory entry with information which you choose in config  |
| [fs_extra::dir::ls](https://docs.rs/fs_extra/*/fs_extra/dir/fn.ls.html)  | Returned collection directory entries with information which you choose in config  |

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
fs_extra = "1.1.0"
```
and this to your crate root:
```rust
extern crate fs_extra;
```
## Examples

The following example shows how to copy a directory recursively and to follow the process.
This example created a directory `dir` contains `test1.txt` file and sub directory `sub`. Folder `sub` inside contains `test2.txt` file.
Then copy `./temp/dir` and all containts to `./out/dir`

```rust
use std::path::Path;
use std::{thread, time};
use std::sync::mpsc::{self, TryRecvError};

extern crate fs_extra;
use fs_extra::dir::*;
use fs_extra::error::*;

fn example_copy() -> Result<()> {

    let path_from = Path::new("./temp");
    let path_to = path_from.join("out");
    let test_folder = path_from.join("test_folder");
    let dir = test_folder.join("dir");
    let sub = dir.join("sub");
    let file1 = dir.join("file1.txt");
    let file2 = sub.join("file2.txt");

    create_all(&sub, true)?;
    create_all(&path_to, true)?;
    fs_extra::file::write_all(&file1, "content1")?;
    fs_extra::file::write_all(&file2, "content2")?;

    assert!(dir.exists());
    assert!(sub.exists());
    assert!(file1.exists());
    assert!(file2.exists());


    let mut options = CopyOptions::new();
    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let handler = |process_info: TransitProcess| {
            tx.send(process_info).unwrap();
            thread::sleep(time::Duration::from_millis(500));
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        };
        copy_with_progress(&test_folder, &path_to, &options, handler).unwrap();
    });

    loop {
        match rx.try_recv() {
            Ok(process_info) => {
                println!("{} of {} bytes",
                         process_info.copied_bytes,
                         process_info.total_bytes);
            }
            Err(TryRecvError::Disconnected) => {
                println!("finished");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
    Ok(())

}
fn main() {
    example_copy();
}
```
