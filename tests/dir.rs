use std::path::{Path, PathBuf};
use std::thread;
use std::sync::mpsc::{self, TryRecvError};
use std::fs::read_dir;

extern crate fs_extra;
use fs_extra::dir::*;
use fs_extra::error::*;



fn files_eq<P, Q>(file1: P, file2: Q) -> bool
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let content1 = fs_extra::file::read_to_string(file1).unwrap();
    let content2 = fs_extra::file::read_to_string(file2).unwrap();
    content1 == content2

}


fn compare_dir<P, Q>(path_from: P, path_to: Q) -> bool
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let mut path_to = path_to.as_ref().to_path_buf();
    match path_from.as_ref().components().last() {
        None => panic!("Invalid folder from"),
        Some(dir_name) => {
            path_to.push(dir_name.as_os_str());
            if !path_to.exists() {
                return false;
            }
        }
    }

    for entry in read_dir(&path_from).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if !compare_dir(path, &path_to) {
                return false;
            }
        } else {
            let mut path_to = path_to.to_path_buf();
            match path.file_name() {
                None => panic!("No file name"),
                Some(file_name) => {
                    path_to.push(file_name);
                    if !path_to.exists() {
                        return false;
                    } else if !files_eq(&path, path_to.clone()) {
                        return false;
                    }
                }
            }
        }
    }

    true
}


const TEST_FOLDER: &'static str = "./tests/temp/dir";



#[test]
fn it_create_all_work() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_all_work");
    test_dir.push("sub_dir");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    assert!(!test_dir.exists());
    create_all(&test_dir, false).unwrap();
    assert!(test_dir.exists());
}

#[test]
fn it_create_work() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_work");
    if !test_dir.exists() {
        create_all(&test_dir, false).unwrap();
    }
    assert!(test_dir.exists());
    test_dir.push("sub_dir");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    create(&test_dir, false).unwrap();
    assert!(test_dir.exists());
}

#[test]
fn it_create_exist_folder() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_exist_folder");
    test_dir.push("sub");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    assert!(!test_dir.exists());
    create_all(&test_dir, false).unwrap();
    assert!(test_dir.exists());
    let mut file_path = test_dir.clone();
    file_path.push("test.txt");
    assert!(!file_path.exists());
    let content = "test_content";
    fs_extra::file::write_all(&file_path, &content).unwrap();
    assert!(file_path.exists());

    match create(&test_dir, false) {
        Ok(_) => panic!("Should be error!"),
        Err(err) => {
            match err.kind {
                ErrorKind::AlreadyExists => {
                    assert!(test_dir.exists());
                    assert!(file_path.exists());
                    let new_content = fs_extra::file::read_to_string(file_path).unwrap();
                    assert_eq!(new_content, content);

                }
                _ => panic!("Wrong error"),
            }
        }

    }
}

#[test]
fn it_create_erase_exist_folder() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_erase_exist_folder");
    test_dir.push("sub");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    assert!(!test_dir.exists());
    create_all(&test_dir, true).unwrap();
    assert!(test_dir.exists());
    let mut file_path = test_dir.clone();
    file_path.push("test.txt");
    assert!(!file_path.exists());
    fs_extra::file::write_all(&file_path, "test_content").unwrap();
    assert!(file_path.exists());

    create(&test_dir, true).unwrap();
    assert!(test_dir.exists());
    assert!(!file_path.exists());
}

#[test]
fn it_create_all_exist_folder() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_all_exist_folder");
    test_dir.push("sub");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    assert!(!test_dir.exists());
    create_all(&test_dir, false).unwrap();
    assert!(test_dir.exists());
    let mut file_path = test_dir.clone();
    file_path.push("test.txt");
    assert!(!file_path.exists());
    let content = "test_content";
    fs_extra::file::write_all(&file_path, &content).unwrap();
    assert!(file_path.exists());

    create_all(&test_dir, false).unwrap();
    assert!(test_dir.exists());
    assert!(file_path.exists());
    let new_content = fs_extra::file::read_to_string(file_path).unwrap();
    assert_eq!(new_content, content);
}

#[test]
fn it_create_all_erase_exist_folder() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_create_all_erase_exist_folder");
    test_dir.push("sub");
    if test_dir.exists() {
        remove(&test_dir).unwrap();
    }
    assert!(!test_dir.exists());
    create_all(&test_dir, true).unwrap();
    assert!(test_dir.exists());
    let mut file_path = test_dir.clone();
    file_path.push("test.txt");
    assert!(!file_path.exists());
    fs_extra::file::write_all(&file_path, "test_content").unwrap();
    assert!(file_path.exists());

    create_all(&test_dir, true).unwrap();
    assert!(test_dir.exists());
    assert!(!file_path.exists());
}

#[test]
fn it_remove_work() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_remove_work");
    test_dir.push("sub");
    test_dir.push("second_sub");
    create_all(&test_dir, true).unwrap();
    assert!(test_dir.exists());
    test_dir.pop();
    test_dir.pop();
    remove(&test_dir).unwrap();
    assert!(!test_dir.exists());
}

#[test]
fn it_remove_not_exist() {
    let mut test_dir = PathBuf::from(TEST_FOLDER);
    test_dir.push("it_remove_not_exist");
    test_dir.push("sub");
    assert!(!test_dir.exists());
    match remove(&test_dir) {
        Ok(_) => {
            assert!(!test_dir.exists());
        }
        Err(err) => panic!(err.to_string()),
    }

}

#[test]
fn it_copy_work() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_work");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let options = CopyOptions::new();
    let result = copy(&path_from, &path_to, &options).unwrap();

    assert_eq!(16, result);
    assert!(path_to.exists());
    assert!(path_from.exists());
    assert!(compare_dir(&path_from, &path_to));
}

#[test]
fn it_copy_not_folder() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_copy_not_folder");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    path_from.push("test.txt");
    fs_extra::file::write_all(&path_from, "test").unwrap();

    match copy(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::InvalidFolder => {
                    let wrong_path = format!("Path \"{}\" is not a directory!",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!("wrong error");
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_copy_source_not_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_copy_source_not_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    assert!(!path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    match copy(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::NotFound => {
                    let wrong_path = format!("Path \"{}\" does not exist",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!(format!("wrong error {}", err.to_string()));
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_copy_exist_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_exist_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert!(files_eq(file1_path, exist_path));
    assert!(path_to.exists());
    assert!(compare_dir(&path_from, &path_to));
}

#[test]
fn it_copy_exist_not_overwrite() {
    let test_name = "sub";
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_copy_exist_not_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());

    let options = CopyOptions::new();
    match copy(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::AlreadyExists => {
                    let wrong_path = format!("Path \"{}\" is exist", exist_path.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!(format!("wrong error {}", err.to_string()));
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_copy_exist_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_exist_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.skip_exist = true;
    copy(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert!(!files_eq(file1_path, &exist_path));
    assert_eq!(fs_extra::file::read_to_string(exist_path).unwrap(),
               exist_content);

    assert!(path_to.exists());
    assert!(!compare_dir(&path_from, &path_to));
}

#[test]
fn it_copy_exist_overwrite_and_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_exist_overwrite_and_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.skip_exist = true;
    copy(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert!(files_eq(file1_path, exist_path));
    assert!(path_to.exists());
    assert!(compare_dir(&path_from, &path_to));
}




#[test]
fn it_copy_progress_work() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_progress_work");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();

    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = copy_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(15, result);
            assert!(path_to.exists());
            assert!(compare_dir(&path_from, &path_to));

        })
        .join();

    loop {
        match rx.try_recv() {
            Ok(process_info) => {
                if process_info.file_name == "test2.txt" {
                    assert_eq!(8, process_info.file_total_bytes);
                    assert_eq!(15, process_info.total_bytes);
                } else if process_info.file_name == "test1.txt" {
                    assert_eq!(7, process_info.file_total_bytes);
                    assert_eq!(15, process_info.total_bytes);
                } else {
                    panic!("Unknow file name!");
                }
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

}

#[test]
fn it_copy_with_progress_not_folder() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_copy_with_progress_not_folder");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    path_from.push("test.txt");
    fs_extra::file::write_all(&path_from, "test").unwrap();
    let func_test = |process_info: TransitProcess| println!("{}", process_info.total_bytes);

    match copy_with_progress(&path_from, &path_to, &options, func_test) {
        Err(err) => {
            match err.kind {
                ErrorKind::InvalidFolder => {
                    let wrong_path = format!("Path \"{}\" is not a directory!",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!("wrong error");
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}


#[test]
fn it_copy_with_progress_work_dif_buf_size() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_with_progress_work_dif_buf_size");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();

    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = copy_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(16, result);
            assert!(path_to.exists());
            assert!(compare_dir(&path_from, &path_to));

            let mut options = CopyOptions::new();
            options.buffer_size = 2;
            options.overwrite = true;
            let (tx, rx) = mpsc::channel();
            let result =
                thread::spawn(move || {
                        let func_test =
                            |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
                        let result = copy_with_progress(&path_from, &path_to, &options, func_test)
                            .unwrap();

                        assert_eq!(16, result);
                        assert!(path_to.exists());
                        assert!(compare_dir(&path_from, &path_to));
                    })
                    .join();
            for i in 1..5 {
                let process_info: TransitProcess = rx.recv().unwrap();
                assert_eq!(i * 2, process_info.file_bytes_copied);
                assert_eq!(i * 2, process_info.copied_bytes);
                assert_eq!(8, process_info.file_total_bytes);
                assert_eq!(16, process_info.total_bytes);
            }
            for i in 1..5 {
                let process_info: TransitProcess = rx.recv().unwrap();
                assert_eq!(i * 2 + 8, process_info.copied_bytes);
                assert_eq!(i * 2, process_info.file_bytes_copied);
                assert_eq!(8, process_info.file_total_bytes);
                assert_eq!(16, process_info.total_bytes);
            }

            match result {
                Ok(_) => {}
                Err(err) => panic!(err),
            }

        })
        .join();

    for i in 1..9 {
        let process_info: TransitProcess = rx.recv().unwrap();
        assert_eq!(i, process_info.file_bytes_copied);
        assert_eq!(i, process_info.copied_bytes);
        assert_eq!(8, process_info.file_total_bytes);
        assert_eq!(16, process_info.total_bytes);
    }
    for i in 1..9 {
        let process_info: TransitProcess = rx.recv().unwrap();
        assert_eq!(i + 8, process_info.copied_bytes);
        assert_eq!(i, process_info.file_bytes_copied);
        assert_eq!(8, process_info.file_total_bytes);
        assert_eq!(16, process_info.total_bytes);
    }

    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
}
#[test]
fn it_copy_with_progress_source_not_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_copy_with_progress_source_not_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    assert!(!path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };

            match copy_with_progress(&path_from, &path_to, &options, func_test) {
                Err(err) => {
                    match err.kind {
                        ErrorKind::NotFound => {
                            let wrong_path = format!("Path \"{}\" does not exist",
                                                     path_from.to_str().unwrap());
                            assert_eq!(wrong_path, err.to_string());
                        }
                        _ => {
                            panic!(format!("wrong error {}", err.to_string()));
                        }
                    }
                }
                Ok(_) => {
                    panic!("should be error");
                }
            }
        })
        .join();
    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => {}
        _ => panic!("should be error"),

    }

}

#[test]
fn it_copy_with_progress_exist_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_with_progress_exist_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();
    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();

    options.buffer_size = 1;
    options.overwrite = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = copy_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(23, result);
            assert!(path_to.exists());
            assert!(compare_dir(&path_from, &path_to));

        })
        .join();


    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => panic!("Errors should not be!"),
        _ => {}

    }

}

#[test]
fn it_copy_with_progress_exist_not_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_with_progress_exist_not_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();

    options.buffer_size = 1;
    let func_test = |process_info: TransitProcess| println!("{}", process_info.total_bytes);
    let result = copy_with_progress(&path_from, &path_to, &options, func_test);
    match result {
        Ok(_) => panic!("Should be error!"),
        Err(err) => {
            match err.kind {
                ErrorKind::AlreadyExists => {}
                _ => panic!("Wrong wrror"),
            }
        }
    }





}

#[test]
fn it_copy_with_progress_exist_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_with_progress_exist_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();

    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();
    options.buffer_size = 1;
    options.skip_exist = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = copy_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(0, result);
            assert!(path_to.exists());
            assert!(!compare_dir(&path_from, &path_to));

        })
        .join();


    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => {}
        _ => panic!("should be error"),

    }

}


#[test]
fn it_copy_with_progress_exist_overwrite_and_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_copy_with_progress_exist_overwrite_and_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();
    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();

    options.buffer_size = 1;
    options.overwrite = true;
    options.skip_exist = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = copy_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(23, result);
            assert!(path_to.exists());
            assert!(compare_dir(&path_from, &path_to));

        })
        .join();


    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
    rx.recv().unwrap();

}




#[test]
fn it_move_work() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_work");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let options = CopyOptions::new();
    let result = move_dir(&path_from, &path_to, &options).unwrap();

    assert_eq!(16, result);
    assert!(path_to.exists());
    assert!(!path_from.exists());
}

#[test]
fn it_move_not_folder() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_move_not_folder");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    path_from.push("test.txt");
    fs_extra::file::write_all(&path_from, "test").unwrap();

    match move_dir(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::InvalidFolder => {
                    let wrong_path = format!("Path \"{}\" is not a directory!",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!("wrong error");
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_move_source_not_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_move_source_not_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    assert!(!path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    match move_dir(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::NotFound => {
                    let wrong_path = format!("Path \"{}\" does not exist",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!(format!("wrong error {}", err.to_string()));
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_move_exist_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_exist_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.overwrite = true;
    move_dir(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert!(path_to.exists());
    assert!(!path_from.exists());
}

#[test]
fn it_move_exist_not_overwrite() {
    let test_name = "sub";
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_move_exist_not_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());

    let options = CopyOptions::new();
    match move_dir(&path_from, &path_to, &options) {
        Err(err) => {
            match err.kind {
                ErrorKind::AlreadyExists => {
                    let wrong_path = format!("Path \"{}\" is exist", exist_path.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!(format!("wrong error {}", err.to_string()));
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}

#[test]
fn it_move_exist_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_exist_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.skip_exist = true;
    move_dir(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert_eq!(fs_extra::file::read_to_string(exist_path).unwrap(),
               exist_content);

    assert!(path_to.exists());
}

#[test]
fn it_move_exist_overwrite_and_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_exist_overwrite_and_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);
    let same_file = "test.txt";

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push(same_file);
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());


    let mut exist_path = path_to.clone();
    exist_path.push(&test_name);
    create(&exist_path, true).unwrap();
    assert!(exist_path.exists());
    exist_path.push(same_file);
    let exist_content = "exist content";
    assert_ne!(exist_content, content1);
    fs_extra::file::write_all(&exist_path, exist_content).unwrap();
    assert!(exist_path.exists());


    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.skip_exist = true;
    move_dir(&path_from, &path_to, &options).unwrap();

    assert!(exist_path.exists());
    assert!(path_to.exists());
    assert!(!path_from.exists());
}


#[test]
fn it_move_progress_work() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_progress_work");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();

    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(15, result);
            assert!(path_to.exists());
            assert!(!path_from.exists());
        })
        .join();

    loop {
        match rx.try_recv() {
            Ok(process_info) => {
                if process_info.file_name == "test2.txt" {
                    assert_eq!(8, process_info.file_total_bytes);
                    assert_eq!(15, process_info.total_bytes);
                } else if process_info.file_name == "test1.txt" {
                    assert_eq!(7, process_info.file_total_bytes);
                    assert_eq!(15, process_info.total_bytes);
                } else {
                    panic!("Unknow file name!");
                }
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

}

#[test]
fn it_move_with_progress_not_folder() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_move_with_progress_not_folder");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    path_from.push("test.txt");
    fs_extra::file::write_all(&path_from, "test").unwrap();
    let func_test = |process_info: TransitProcess| println!("{}", process_info.total_bytes);

    match move_dir_with_progress(&path_from, &path_to, &options, func_test) {
        Err(err) => {
            match err.kind {
                ErrorKind::InvalidFolder => {
                    let wrong_path = format!("Path \"{}\" is not a directory!",
                                             path_from.to_str().unwrap());
                    assert_eq!(wrong_path, err.to_string());
                }
                _ => {
                    panic!("wrong error");
                }
            }
        }
        Ok(_) => {
            panic!("should be error");
        }
    }
}


#[test]
fn it_move_with_progress_work_dif_buf_size() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_with_progress_work_dif_buf_size");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content1";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();

    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(16, result);
            assert!(path_to.exists());
            assert!(!path_from.exists());

            create_all(&path_from, true).unwrap();
            assert!(path_from.exists());
            let mut file1_path = path_from.clone();
            file1_path.push("test1.txt");
            let content1 = "content1";
            fs_extra::file::write_all(&file1_path, &content1).unwrap();
            assert!(file1_path.exists());

            let mut sub_dir_path = path_from.clone();
            sub_dir_path.push("sub");
            create(&sub_dir_path, true).unwrap();
            let mut file2_path = sub_dir_path.clone();
            file2_path.push("test2.txt");
            let content2 = "content2";
            fs_extra::file::write_all(&file2_path, &content2).unwrap();
            assert!(file2_path.exists());

            let mut options = CopyOptions::new();
            options.buffer_size = 2;
            options.overwrite = true;
            let (tx, rx) = mpsc::channel();
            let result =
                thread::spawn(move || {
                        let func_test =
                            |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
                        let result =
                            move_dir_with_progress(&path_from, &path_to, &options, func_test)
                                .unwrap();

                        assert_eq!(16, result);
                        assert!(path_to.exists());
                        assert!(!path_from.exists());
                    })
                    .join();
            for i in 1..5 {
                let process_info: TransitProcess = rx.recv().unwrap();
                assert_eq!(i * 2, process_info.file_bytes_copied);
                assert_eq!(i * 2, process_info.copied_bytes);
                assert_eq!(8, process_info.file_total_bytes);
                assert_eq!(16, process_info.total_bytes);
            }
            for i in 1..5 {
                let process_info: TransitProcess = rx.recv().unwrap();
                assert_eq!(i * 2 + 8, process_info.copied_bytes);
                assert_eq!(i * 2, process_info.file_bytes_copied);
                assert_eq!(8, process_info.file_total_bytes);
                assert_eq!(16, process_info.total_bytes);
            }

            match result {
                Ok(_) => {}
                Err(err) => panic!(err),
            }

        })
        .join();

    for i in 1..9 {
        let process_info: TransitProcess = rx.recv().unwrap();
        assert_eq!(i, process_info.file_bytes_copied);
        assert_eq!(i, process_info.copied_bytes);
        assert_eq!(8, process_info.file_total_bytes);
        assert_eq!(16, process_info.total_bytes);
    }
    for i in 1..9 {
        let process_info: TransitProcess = rx.recv().unwrap();
        assert_eq!(i + 8, process_info.copied_bytes);
        assert_eq!(i, process_info.file_bytes_copied);
        assert_eq!(8, process_info.file_total_bytes);
        assert_eq!(16, process_info.total_bytes);
    }

    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
}
#[test]
fn it_move_with_progress_source_not_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    path_from.push("it_move_with_progress_source_not_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push("sub");

    assert!(!path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let options = CopyOptions::new();
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };

            match move_dir_with_progress(&path_from, &path_to, &options, func_test) {
                Err(err) => {
                    match err.kind {
                        ErrorKind::NotFound => {
                            let wrong_path = format!("Path \"{}\" does not exist",
                                                     path_from.to_str().unwrap());
                            assert_eq!(wrong_path, err.to_string());
                        }
                        _ => {
                            panic!(format!("wrong error {}", err.to_string()));
                        }
                    }
                }
                Ok(_) => {
                    panic!("should be error");
                }
            }
        })
        .join();
    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => {}
        _ => panic!("should be error"),

    }
}

#[test]
fn it_move_with_progress_exist_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_with_progress_exist_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();
    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();

    options.buffer_size = 1;
    options.overwrite = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(23, result);
            assert!(path_to.exists());
            assert!(!path_from.exists());

        })
        .join();


    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
    rx.recv().unwrap();

}

#[test]
fn it_move_with_progress_exist_not_overwrite() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_with_progress_exist_not_overwrite");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();

    options.buffer_size = 1;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test);
            match result {
                Ok(_) => panic!("Should be error!"),
                Err(err) => {
                    match err.kind {
                        ErrorKind::AlreadyExists => {}
                        _ => panic!("Wrong wrror"),
                    }
                }
            }

        })
        .join();

    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => {}
        _ => panic!("should be error"),

    }
}

#[test]
fn it_move_with_progress_exist_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_with_progress_exist_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();

    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();
    options.buffer_size = 1;
    options.skip_exist = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {
            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(0, result);
            assert!(path_to.exists());

        })
        .join();


    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }

    match rx.recv() {
        Err(_) => {}
        _ => panic!("should be error"),

    }

}


#[test]
fn it_move_with_progress_exist_overwrite_and_skip_exist() {
    let mut path_from = PathBuf::from(TEST_FOLDER);
    let test_name = "sub";
    path_from.push("it_move_with_progress_exist_overwrite_and_skip_exist");
    let mut path_to = path_from.clone();
    path_to.push("out");
    path_from.push(&test_name);

    create_all(&path_from, true).unwrap();
    assert!(path_from.exists());
    create_all(&path_to, true).unwrap();
    assert!(path_to.exists());

    let mut file1_path = path_from.clone();
    file1_path.push("test1.txt");
    let content1 = "content";
    fs_extra::file::write_all(&file1_path, &content1).unwrap();
    assert!(file1_path.exists());

    let mut sub_dir_path = path_from.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2_path = sub_dir_path.clone();
    file2_path.push("test2.txt");
    let content2 = "content2";
    fs_extra::file::write_all(&file2_path, &content2).unwrap();
    assert!(file2_path.exists());

    let mut options = CopyOptions::new();
    copy(&path_from, &path_to, &options).unwrap();
    fs_extra::file::write_all(&file2_path, "another conntent").unwrap();

    options.buffer_size = 1;
    options.overwrite = true;
    options.skip_exist = true;
    let (tx, rx) = mpsc::channel();
    let result = thread::spawn(move || {

            let func_test = |process_info: TransitProcess| { tx.send(process_info).unwrap(); };
            let result = move_dir_with_progress(&path_from, &path_to, &options, func_test).unwrap();

            assert_eq!(23, result);
            assert!(path_to.exists());
            assert!(!path_from.exists());

        })
        .join();



    match result {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
    rx.recv().unwrap();

}


#[test]
fn it_get_folder_size() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_folder_size");
    path.push("dir");

    create_all(&path, true).unwrap();
    assert!(path.exists());

    let mut file1 = path.clone();
    file1.push("test1.txt");
    fs_extra::file::write_all(&file1, "content1").unwrap();
    assert!(file1.exists());

    let mut sub_dir_path = path.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2 = sub_dir_path.clone();
    file2.push("test2.txt");
    fs_extra::file::write_all(&file2, "content2").unwrap();
    assert!(file2.exists());

    let result = get_size(&path).unwrap();

    assert_eq!(16, result);
}

#[test]
fn it_get_file_size() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_file_size");

    create_all(&path, true).unwrap();
    assert!(path.exists());

    let mut file = path.clone();
    file.push("test1.txt");
    fs_extra::file::write_all(&file, "content").unwrap();
    assert!(file.exists());

    let result = get_size(&path).unwrap();

    assert_eq!(7, result);
}

#[test]
fn it_get_size_not_found() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_size_not_found");

    assert!(!path.exists());

    match get_size(&path) {
        Ok(_) => panic!("Should be a error!"),
        Err(err) => {
            match err.kind {
                ErrorKind::NotFound => {}
                _ => panic!("Wrong error!"),
            }
        }
    };

}


#[test]
fn it_get_dir_content() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_dir_content");
    path.push("dir");

    create_all(&path, true).unwrap();
    assert!(path.exists());

    let mut file1 = path.clone();
    file1.push("test1.txt");
    fs_extra::file::write_all(&file1, "content1").unwrap();
    assert!(file1.exists());

    let mut sub_dir_path = path.clone();
    sub_dir_path.push("sub");
    create(&sub_dir_path, true).unwrap();
    let mut file2 = sub_dir_path.clone();
    file2.push("test2.txt");
    fs_extra::file::write_all(&file2, "content2").unwrap();
    assert!(file2.exists());

    let result = get_dir_content(&path).unwrap();

    assert_eq!(16, result.dir_size);
    assert_eq!(2, result.files.len());
    assert_eq!(2, result.directories.len());

    let dir1 = file1.parent().unwrap().to_str().unwrap().to_string();
    let dir2 = file2.parent().unwrap().to_str().unwrap().to_string();
    let file1 = file1.to_str().unwrap().to_string();
    let file2 = file2.to_str().unwrap().to_string();

    let mut files_correct = true;
    for file in result.files {
        if file != file1 && file != file2 {
            files_correct = false;
        }
    }
    assert!(files_correct);

    let mut directories_correct = true;
    for dir in result.directories {
        if dir != dir1 && dir != dir2 {
            directories_correct = false;
        }
    }
    assert!(directories_correct);
}


#[test]
fn it_get_dir_content_path_file() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_dir_content_path_file");

    create_all(&path, true).unwrap();
    assert!(path.exists());

    let mut file = path.clone();
    file.push("test1.txt");
    fs_extra::file::write_all(&file, "content1").unwrap();
    assert!(file.exists());

    let result = get_dir_content(&file).unwrap();

    assert_eq!(8, result.dir_size);
    assert_eq!(1, result.files.len());
    assert_eq!(0, result.directories.len());
    assert_eq!(file.to_str().unwrap().to_string(), result.files[0]);
}

#[test]
fn it_get_dir_content_not_found() {
    let mut path = PathBuf::from(TEST_FOLDER);
    path.push("it_get_dir_content_not_found");

    assert!(!path.exists());


    match get_dir_content(&path) {
        Ok(_) => panic!("Should be a error!"),
        Err(err) => {
            match err.kind {
                ErrorKind::NotFound => {}
                _ => panic!("Wrong error!"),
            }
        }
    }

}
