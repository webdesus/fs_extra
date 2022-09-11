extern crate chrono;
extern crate filetime;

use std::collections::BTreeMap;
// use std::io::{ErrorKind, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

extern crate fs_extra;
extern crate yaml_rust;
use filetime::FileTime;
use fs_extra::error::*;
use fs_extra::file::*;
use fs_extra::dir;

use yaml_rust::{Yaml, YamlLoader};
use chrono::DateTime;

struct Test {
    path: PathBuf,
}

struct Ast {
    name: Option<String>,
    files: Files,
    attrs: Attrs,
}

struct Files {
    files: BTreeMap<String, File>,
}

struct Attrs {
    attrs: Vec<(PathBuf, Attr)>,
}

enum Attr {
    PermissionsUnix(u32),
    Readonly(bool),
    MTime(FileTime),
    ATime(FileTime),
}

enum File {
    Dir { content: Files },
    File { content: String },
}

impl Ast {
    fn parse_doc(doc: &Yaml) -> Self {
        let name = doc["NAME"].as_str().map(|s| s.to_owned());
        let files = Files::parse(&doc["FILES"]);
        let attrs = Attrs::parse(&doc["ATTRIBUTES"])
        Ast {
            name,
            files,
            attrs,
        }
    }

    fn initialize(&self, path: &Path) {
        self.files.initialize(path);
        self.attrs.initialize(path);
    }

    fn check(&self, path: &Path) {
        self.files.check(path);
        self.attrs.check(path);
    }
}

impl Files {
    fn parse(doc: &Yaml) -> Self {
        if doc.is_badvalue() {
            panic!("invalid FILES: badvalue: expected hash at FILES");
        }
        let hash = doc.as_hash().expect("invalid FILES: expected hash at FILES");
        let mut result = Files {
            files: BTreeMap::new(),
        };
        for (key, value) in hash {
            let name = key.as_str().expect("expected file name");
            let file = match value {
                &Yaml::String(ref file_content) => {
                    File::File { content: file_content.to_owned() }
                }
                hash @ &Yaml::Hash(..) => {
                    File::Dir { content: Files::parse(hash) }
                }
                other => {
                    panic!("invalid yaml value, expected string or hash, found {:?}", other);
                }
            };
            result.files.insert(name.to_owned(), file);
        }
        result
    }

    fn initialize(&self, path: &Path) {
        for (name, file) in &self.files {
            let path = &*path.join(name);
            match file {
                &File::Dir { ref content } => {
                    Files::create_dir(path);
                    content.initialize(path);
                }
                &File::File { ref content } => {
                    Files::create_file(path, &content[..])
                }
            }
        }
    }

    fn check(&self, path: &Path) {
        for (name, file) in &self.files {
            let path = &*path.join(name);
            match file {
                &File::Dir { ref content } => {
                    Files::check_dir(path);
                    content.check(path);
                }
                &File::File { ref content } => {
                    Files::check_file(path, &content[..])
                }
            }
        }
    }

    fn create_dir(path: &Path) {
        dir::create_all(path, true).expect("file structure initialization failed");
    }

    fn create_file(path: &Path, file_content: &str) {
        write_all(path, file_content).expect("file structure initialization failed");
    }

    fn check_dir(path: &Path) {
        if path.is_dir() {
            // ok
        } else {
            panic!("expected directory not found at {:?}", path);
        }
    }

    fn check_file(path: &Path, expected_file_content: &str) {
        let actual_content = read_to_string(path).ok();
        assert_eq!(
            actual_content.map(|s| &s[..]),
            Some(expected_file_content),
            "file content mismatch or file does not exist at {:?}",
            path
        );
    }
}

impl Attrs {
    fn parse(doc: &Yaml) -> Self {
        if doc.is_badvalue() {
            panic!("missing ATTRIBUTES");
        }
        let attrs = doc.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes");
        let attrs = attrs.iter().flat_map(|(path_str, attr_hash)| {
            let inner_attrs = attr_hash.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes");
            inner_attrs.iter().map(|(attr_name, val)| {
                let attr = match attr_name.as_str().expect("invalid attribute name: expected string (`permissions`, `mtime`, `atime`, `readonly`)") {
                    "permissions" => {
                        Attr::parse_permissions(val)
                    }
                    "readonly" => {
                        Attr::parse_readonly(val)
                    }
                    "mtime" => {
                        Attr::parse_mtime(val)
                    }
                    "atime" => {
                        Attr::parse_atime(val)
                    }
                };
                let path = path_str.as_str().expect("invalid ATTRIBUTES: expected hash of hashes").into();
                (path, attr)
            })
        }).collect();
        Attrs { attrs }
    }

    fn initialize(&self, path: &Path) {
        for &(ref inner_path, ref attr) in &self.attrs {
            let full_path = path.join(inner_path);
            match attr {
                &Attr::PermissionsUnix(perm) => {
                    #[cfg(unix)] {
                        use std::os::unix::fs::PermissionsExt;
                        fs::metadata(full_path).expect("failed setting permissions: entry does not exist").permissions().set_mode(perm)
                    }
                }
                &Attr::Readonly(perm) => {
                    #[cfg(not(unix))] {
                        fs::metadata(full_path).expect("failed setting permissions: entry does not exist").permissions().set_readonly(perm)
                    }
                }
                &Attr::MTime(mtime) => {
                    ::filetime::set_file_mtime(full_path, mtime).expect("failed setting mtime");
                }
                &Attr::ATime(atime) => {
                    ::filetime::set_file_atime(full_path, atime).expect("failed setting atime");
                }
            }
        }
    }

    fn check(&self, path: &Path) {
        for &(ref inner_path, ref attr) in &self.attrs {
            let full_path = path.join(inner_path);
            match attr {
                &Attr::PermissionsUnix(expected_perm) => {
                    Attrs::check_permissions(full_path, expected_perm);
                }
                &Attr::Readonly(expected_readonly) => {
                    Attrs::check_readonly(full_path, expected_readonly);
                }
                &Attr::MTime(mtime) => {
                    ::filetime::set_file_mtime(full_path, mtime).expect("failed setting mtime");
                }
                &Attr::ATime(atime) => {
                    ::filetime::set_file_atime(full_path, atime).expect("failed setting atime");
                }
            }
        }
    }

    #[cfg(unix)]
    fn check_permissions(full_path: PathBuf, expected_perm: u32) {
        use std::os::unix::fs::PermissionsExt;
        let actual_perm = fs::metadata(full_path).expect("failed checking permissions: entry does not exist").permissions();
        assert_eq!(actual_perm.mode(), expected_perm);
    }

    #[cfg(not(unix))]
    fn check_permissions(full_path: PathBuf, expected_perm: u32) {
        // nothing to do
    }

    #[cfg(unix)]
    fn check_readonly(full_path: PathBuf, expected_perm: bool) {
        // nothing to do
    }

    #[cfg(not(unix))]
    fn check_readonly(full_path: PathBuf, expected_readonly: bool) {
        let actual_perm = fs::metadata(full_path).expect("failed checking permissions: entry does not exist").readonly();
        assert_eq!(actual_perm, expected_readonly);
    }
}

impl Attr {
    fn parse_mtime(value: &Yaml) -> Self {
        let value = value.as_str().expect("invalid attr `mtime` value: expected string");
        Attr::MTime(Attr::parse_time(value))
    }

    fn parse_atime(value: &Yaml) -> Self {
        let value = value.as_str().expect("invalid attr `atime` value: expected string");
        Attr::ATime(Attr::parse_time(value))
    }

    fn parse_permissions(value: &Yaml) -> Self {
        let value = value.as_str().expect("invalid attr `permissions` value: expected string (valid: drwxrwxrwx)");
        assert_eq!("-rw-rw-r--".len(), value.len(), "invalid attr `permissions` value: invalid length (valid: drwxrwxrwx)");
        let mode = value.bytes().zip("drwxrwxrwx".bytes()).rev().enumerate().map(|(i, (actual, positive))| {
            if actual == positive {
                1 << i
            } else if actual == b'-' {
                0
            } else {
                panic!("invalid attr `permissions` value: invalid content (valid: drwxrwxrwx)")
            }
        }).sum();
        Attr::PermissionsUnix(mode)
    }

    fn parse_readonly(value: &Yaml) -> Self {
        Attr::Readonly(value.as_bool().expect("invalid attr `readonly` value: expected boolean"))
    }

    fn parse_time(value: &str) -> FileTime {
        let date_time = DateTime::parse_from_rfc3339(value).expect("atime/mtime attr: expected date");
        FileTime::from_unix_time(date_time.timestamp(), date_time.timestamp_subsec_nanos())
    }
}

impl Test {
    pub fn new(description: &str) -> Self {
        const TEST_FOLDER: &'static str = "./tests/temp/file";

        let mut this = Test {
            path: PathBuf::from(TEST_FOLDER),
        };
        this.initialize(description);
        this
    }

    fn create_dir(path: &Path, dir_content: &Yaml) {
        if dir_content.is_badvalue() {
            panic!("invalid FILES: expected hash at FILES");
        }
        let hash = dir_content.as_hash().expect("expected hash");
        for (key, value) in hash {
            let name = key.as_str().expect("expected file name");
            match value {
                &Yaml::String(ref file_content) => {
                    Test::create_file(&*path.join(name), &file_content[..]);
                }
                hash @ &Yaml::Hash(..) => {
                    Test::create_dir(&*path.join(name), hash);
                }
                other => {
                    panic!("invalid yaml value, expected string or hash, found {:?}", other);
                }
            }
        }
    }

    fn create_file(path: &Path, file_content: &str) {
        write_all(path, file_content).expect("file structure initialization failed");
    }

    fn set_attributes(root: &Path, attributes: Yaml) {
        for (filepath, attr_hash) in attributes.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes") {
            let subpath = filepath.as_str().expect("invalid AT<TRIBUTES: expected string key");
            let full_path = root.join(PathBuf::from(subpath));
            assert!(full_path.exists());
            for (attr, value) in attr_hash.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes") {
                let value = value.as_str().expect("invalid attribute value: expected string");
                match attr.as_str().expect("invalid attribute name: expected string (`permissions`, `mtime`, `atime`)") {
                    "permissions" => {

                    }
                    ty @ ("mtime" | "atime") => {

                        let date_time = DateTime::parse_from_rfc3339(value).expect("ATTRIBUTES: expected date");
                        let file_time = FileTime::from_unix_time(date_time.timestamp(), date_time.timestamp_subsec_nanos());
                        if ty == "mtime" {
                            ::filetime::set_file_mtime(full_path, file_time).expect("failed setting mtime");
                        } else {
                            ::filetime::set_file_atime(full_path, file_time).expect("failed setting atime");
                        }
                    }
                    other => {
                        panic!("unknown attribute: expected `permissions`, `mtime`, `atime`, found `{}`", other);
                    }
                }
            }
        }
    }

    fn check_dir(path: &Path, dir_content: &Yaml) {
        dir::create_all(path, true).expect("file structure initialization failed");
        if dir_content.is_badvalue() {
            panic!("invalid FILES: expected hash at FILES");
        }
        let hash = dir_content.as_hash().expect("expected hash");
        for (key, value) in hash {
            let name = key.as_str().expect("expected file name");
            match value {
                &Yaml::String(ref file_content) => {
                    Test::create_file(&*path.join(name), &file_content[..]);
                }
                hash @ &Yaml::Hash(..) => {
                    Test::create_dir(&*path.join(name), hash);
                }
                other => {
                    panic!("invalid yaml value, expected string or hash, found {:?}", other);
                }
            }
        }
    }

    fn check_attributes(path: &Path, attributes: &[Yaml]) {
        for attr in attributes {
            let attrs = attr.as_hash().expect("invalid ATTRIBUTES: expected vec of hashes");
            for (attr, value) in attrs {
                let subpath = key.as_str().expect("invalid ATTRIBUTES: expected string key");
                let full_path = path.join(PathBuf::from(subpath));
                match attr.as_str().expect("invalid attribute name: expected string (`permissions`, `mtime`, `atime`)") {
                    "permissions" => {

                    }
                    ty @ ("mtime" | "atime") => {

                    }
                }
            }
        }
    }

    fn initialize(&self, description: &str) {
        let docs = YamlLoader::load_from_str(description).expect("failed to load yaml");
        let doc = &docs[0];
        let ast = Ast::parse_doc(doc);
        ast.initialize(&*self.path);
        Test::set(ast);
        let name = doc["NAME"].as_str().expect("invalid NAME: expected string at NAME");
        Test::create_dir(&*self.path, &doc["FILES"]);
        let attributes = doc["ATTRIBUTES"];
        if !attributes.is_badvalue() {
            Test::set_attributes(&*self.path, attributes)
        }
    }

    pub fn check(&self, description: &str) {
        let docs = YamlLoader::load_from_str(description).expect("failed to load yaml");
        let doc = &docs[0];
        let ast = Ast::parse_doc(doc);
        ast.check(&*self.path);
        let name = doc["NAME"].as_str().expect("invalid NAME: expected string at NAME");
        Test::check_dir(&*self.path, &doc["FILES"]);
        let attributes = &doc["ATTRIBUTES"];
        if !attributes.is_badvalue() {
            Test::check_attributes(&*self.path, &attributes.as_vec().expect("invalid ATTRIBUTES: expected vec of hashes")[..])
        }
    }

    pub fn file(&self, path: &str) -> PathBuf {
        self.path.join(PathBuf::from(path))
    }
}

#[test]
fn it_read_and_write_work() {
    let test = Test::new(r#"
        NAME: it_read_and_write_work
        FILES:
            dir:
                test.txt: test_0
    "#);

    const CONTENT1: &'static str = "test_1";
    const CONTENT2: &'static str = "test_2";

    let test_file = test.file("dir/test.txt");
    assert!(test_file.exists());

    write_all(&test_file, CONTENT1).unwrap();
    let read1 = read_to_string(&test_file).unwrap();
    assert_eq!(CONTENT1, read1);
    write_all(&test_file, CONTENT2).unwrap();
    let read2 = read_to_string(&test_file).unwrap();
    assert_eq!(CONTENT2, read2)
}

#[test]
fn it_set_attributes() {
    let test = Test::new(r#"
        NAME: it_read_and_write_work
        FILES:
            dir:
                test.txt: test_0
        ATTRIBUTES:
            dir/test.txt:
                permissions: -rw-rw-r--
                readonly: false
                mtime: 2021-06-09 08:02:32+02:00
                atime: 2022-07-10 08:02:32+02:00
    "#);

    const CONTENT1: &'static str = "test_1";
    const CONTENT2: &'static str = "test_2";

    let test_file = test.file("dir/test.txt");
    assert!(test_file.exists());

    write_all(&test_file, CONTENT1).unwrap();
    let read1 = read_to_string(&test_file).unwrap();
    assert_eq!(CONTENT1, read1);
    write_all(&test_file, CONTENT2).unwrap();
    let read2 = read_to_string(&test_file).unwrap();
    assert_eq!(CONTENT2, read2)
}

#[test]
fn it_read_not_exist_file() {
    let test = Test::new(r#"
        NAME: it_read_not_exist_file
        FILES:
            dir: {}
    "#);

    let test_file = test.file("dir/test.txt");
    assert!(!test_file.exists());
    match read_to_string(&test_file) {
        Ok(_) => panic!("should be error"),
        Err(err) => match err.kind {
            ErrorKind::NotFound => {}
            _ => panic!("wrong error"),
        },
    }
}