# fs_extra

A Rust library for more work functionality with file system.

[![Build Status](https://travis-ci.org/webdesus/fs_extra.svg)](https://travis-ci.org/webdesus/fs_extra)
[![Crates.io Status](https://img.shields.io/crates/v/fs_extra.svg)](https://crates.io/crates/fs_extra)
[![Docs](https://docs.rs/fs_extra/badge.svg)](https://docs.rs/fs_extra)


## Key features:

* Copy files with recept information about process.

* Copy directory recursively and recept information about process.

* Move files with recept information about process.

* Move directories recursively and recept information about process.

* One method for create and write `String` content in file.

* One method for open and read `String` content from file.

## Functions:

| Function | Description |
| ------------- | ------------- |
| [fs_extra::fs::copy](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.copy.html)  | Copies the contents of one file to another |
| [fs_extra::fs::copy_with_progress](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.copy_with_progress.html)  | Copies the contents of one file to another with recept information about process  |
| [fs_extra::fs::move_file](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.move_file.html)  | Moves file from one place to another  |
| [fs_extra::fs::move_file_with_progress](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.move_file_with_progress.html)  | Moves file from one place to another with recept information about process  |
| [fs_extra::fs::remove](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.remove.html)  | Removes a file from the filesystem  |
| [fs_extra::fs::read_to_string](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.read_to_string.html)  | Read file content, placing him into `String`  |
| [fs_extra::fs::write_all](https://docs.rs/fs_extra/0.1.0/fs_extra/file/fn.write_all.html)  | Write `String` content into inside target file  |
| [fs_extra::dir::create](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.create.html)  | Creates a new, empty directory at the provided path  |
| [fs_extra::dir::create_all](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.create_all.html)  | Recursively create a directory and all of its parent components if they are missing  |
| [fs_extra::dir::copy](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.copy.html)  | Copies the directory contents from one place to another using recursive method  |
| [fs_extra::dir::copy_with_progress](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.copy_with_progress.html)  | Copies the directory contents from one place to another using recursive method, with recept information about process]()  |
| [fs_extra::dir::move_dir](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.move_dir.html)  | Moves the directory contents from one place to another  |
| [fs_extra::dir::move_dir_with_progress](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.move_dir_with_progress.html)  | Moves the directory contents from one place to another with recept information about process  |
| [fs_extra::dir::remove](https://docs.rs/fs_extra/0.1.0/fs_extra/dir/fn.remove.html)  | Removes directory  |



## Examples

The following example shows how to copy a directory recursively and to follow the process.
This example created a directory `dir` contains `test1.txt` file and sub directory `sub`. Folder `sub` inside contains `test2.txt` file.
Then copy `./temp/dir` and all containts to `./out/dir`

```rust
 use std::path::{Path, PathBuf};
 use std::{thread, time};
 use std::sync::mpsc::{self, TryRecvError};

 extern crate fs_extra;
 use fs_extra::dir::*;
 use fs_extra::error::*;

 fn example_copy() -> Result<()> {

     let mut path_from = PathBuf::from("./temp");
     let test_name = "dir";
     path_from.push("test_folder");
     let mut path_to = path_from.clone();
     path_to.push("out");
     path_from.push(&test_name);

     create_all(&path_from, true)?;
     assert!(path_from.exists());
     create_all(&path_to, true)?;
     assert!(path_to.exists());

     let mut file1_path = path_from.clone();
     file1_path.push("test1.txt");
     let content1 = "content";
     fs_extra::file::write_all(&file1_path, &content1)?;
     assert!(file1_path.exists());

     let mut sub_dir_path = path_from.clone();
     sub_dir_path.push("sub");
     create(&sub_dir_path, true)?;
     let mut file2_path = sub_dir_path.clone();
     file2_path.push("test2.txt");
     let content2 = "content2";
     fs_extra::file::write_all(&file2_path, &content2)?;
     assert!(file2_path.exists());

     let mut options = CopyOptions::new();

     options.buffer_size = 1;
     let (tx, rx) = mpsc::channel();
     thread::spawn(move || {
         let handler = |process_info: TransitProcess| {
             tx.send(process_info).unwrap();
             thread::sleep(time::Duration::from_millis(500));
         };
         copy_with_progress(&path_from, &path_to, &options, handler).unwrap();
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
