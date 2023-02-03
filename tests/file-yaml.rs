extern crate chrono;
extern crate filetime;

use std::collections::{BTreeMap, HashSet};
// use std::io::{ErrorKind, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

extern crate fs_extra;
extern crate yaml_rust;
extern crate linked_hash_map;
use filetime::FileTime;
use fs_extra::error::*;
use fs_extra::file::*;
use fs_extra::dir;
use linked_hash_map::LinkedHashMap;

use yaml_rust::{Yaml, YamlLoader, YamlEmitter};
use chrono::{DateTime, Utc, NaiveDateTime, FixedOffset};

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
        let attrs = if doc["ATTRIBUTES"].is_badvalue() {
            Attrs::new()
        } else {
            Attrs::parse(&doc["ATTRIBUTES"])
        };
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

    fn from_fs(name: String, path: &Path) -> Self {
        Ast {
            name: Some(name),
            files: Files::from_fs(path),
            attrs: Attrs::from_fs(path),
        }
    }

    fn dump(&self, root_path: &Path) -> String {
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&self.into_yaml_doc(root_path)).unwrap(); // dump the YAML object to a String
        out_str
    }

    fn into_yaml_doc(&self, root_path: &Path) -> Yaml {
        let mut hash = LinkedHashMap::new();
        hash.insert(Yaml::String("NAME".into()), Yaml::String(self.name.clone().unwrap()));
        hash.insert(Yaml::String("FILES".into()), self.files.into_yaml());
        hash.insert(Yaml::String("ATTRIBUTES".into()), self.attrs.into_yaml(root_path));
        Yaml::Hash(hash)
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
                    println!("name {:?} dir {:?}", name, path);
                    Files::create_dir(path);
                    content.initialize(path);
                }
                &File::File { ref content } => {
                    println!("file {:?}", path);
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
            actual_content,
            Some(expected_file_content.to_owned()),
            "file content mismatch or file does not exist at {:?}",
            path
        );
    }

    fn from_fs(path: &Path) -> Self {
        use fs_extra::dir::{DirEntryAttr, DirEntryValue, ls};
        let mut config = HashSet::new();
        config.insert(DirEntryAttr::Name);
        config.insert(DirEntryAttr::BaseInfo);
        
        let result = ls(path, &config).expect("invalid path");
        if matches!(result.base[&DirEntryAttr::IsDir], DirEntryValue::Boolean(true)) {

        }
        for entry in result.items {
            unimplemented!()
        }
        unimplemented!()
    }

    fn into_yaml(&self) -> Yaml {
        Yaml::Hash(
            self.files.iter().map(|(filename, content)| {
                let val = match content {
                    File::Dir { content } => {
                        content.into_yaml()
                    }
                    File::File { content } => {
                        Yaml::String(content.clone())
                    }
                };
                (Yaml::String(filename.clone()), val)
            }).collect()
        )
    }
}

const INVALID_ATTRIBUTE_MSG: &'static str = "invalid attribute name: expected string (`permissions`, `mtime`, `atime`, `readonly`)";

impl Attrs {
    fn new() -> Self {
        Attrs { attrs: vec![] }
    }

    fn parse(doc: &Yaml) -> Self {
        if doc.is_badvalue() {
            panic!("missing ATTRIBUTES");
        }
        let attrs = doc.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes");
        let attrs = attrs.iter().flat_map(|(path_str, attr_hash)| {
            let inner_attrs = attr_hash.as_hash().expect("invalid ATTRIBUTES: expected hash of hashes");
            let path: PathBuf = path_str.as_str().expect("invalid ATTRIBUTES: expected hash of hashes").into();
            inner_attrs.iter().map(move |(attr_name, val)| {
                let attr = match attr_name.as_str().expect(INVALID_ATTRIBUTE_MSG) {
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
                    _ => {
                        panic!("{}", INVALID_ATTRIBUTE_MSG)
                    }
                };
                (path.clone(), attr)
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

    fn permissions_to_string(mode: u32) -> String {
        "drwxrwxrwx".bytes().rev().map(|letter| {
            let bit = mode % 2;
            mode /= 2;
            if bit == 1 { letter as char } else { '-' }
        }).collect::<String>().chars().rev().collect::<String>()
    }

    #[cfg(unix)]
    fn check_permissions(full_path: PathBuf, expected_perm: u32) {
        use std::os::unix::fs::PermissionsExt;
        let actual_perm = fs::metadata(full_path).expect("failed checking permissions: entry does not exist").permissions();
        assert_eq!(Attrs::permissions_to_string(actual_perm.mode()), Attrs::permissions_to_string(expected_perm));
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

    fn into_yaml(&self, root_path: &Path) -> Yaml {
        let attrs_by_str = BTreeMap::new();
        for (path, attr) in self.attrs.iter() {
            let key = path.strip_prefix(root_path).expect("prefix path mismatch").to_str().expect("invalid unicode path").to_string();
            attrs_by_str.entry(key).or_insert(vec![]).push(attr);
        }
        Yaml::Hash(
            attrs_by_str.into_iter().map(|(path, attrs)| {
                (
                    Yaml::String(path),
                    Yaml::Hash(
                        attrs.iter().map(|attr| attr.into_yaml()).collect()
                    )
                )
            }).collect()
        )
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
        assert_eq!(date_time.timezone().local_minus_utc(), 0, "invalid timezone: expected UTC");
        FileTime::from_unix_time(date_time.timestamp(), date_time.timestamp_subsec_nanos())
    }

    fn into_yaml(&self) -> (Yaml, Yaml) {
        let val = match self {
            &Attr::Readonly(readonly) => {
                Yaml::Boolean(readonly)
            }
            Attr::ATime(ref time) | Attr::MTime(ref time) => {
                Yaml::String(
                    DateTime::<FixedOffset>::from_utc(
                        NaiveDateTime::from_timestamp(time.unix_seconds(), time.nanoseconds()), FixedOffset::east(0)
                    ).to_rfc3339()
                )
            }
            &Attr::PermissionsUnix(mode) => {
                Yaml::String(Attrs::permissions_to_string(mode))
            }
        };
        (Yaml::String(self.name().into()), val)
    }

    fn name(&self) -> &'static str {
        match self {
            Attr::Readonly(..) => "readonly",
            Attr::ATime(..) => "atime",
            Attr::MTime(..) => "mtime",
            Attr::PermissionsUnix(..) => "permissions",
        }
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

    fn initialize(&mut self, description: &str) {
        let docs = YamlLoader::load_from_str(description).expect("failed to load yaml");
        let doc = &docs[0];
        let ast = Ast::parse_doc(doc);
        self.path = self.path.join(ast.name.as_ref().unwrap());
        ast.initialize(&*self.path);
    }

    pub fn check(&self, description: &str) {
        let docs = YamlLoader::load_from_str(description).expect("failed to load yaml");
        let doc = &docs[0];
        let ast = Ast::parse_doc(doc);
        ast.check(&*self.path);
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
    assert_eq!(CONTENT2, read2);

    test.check(r"
        FILES:
            dir:
                test.txt: test_2
    ");
}

#[test]
fn it_set_attributes() {
    let test = Test::new(r#"
        NAME: it_set_attributes
        FILES:
            dir:
                test.txt: test_0
        ATTRIBUTES:
            dir/test.txt:
                permissions: -rw-rw-r--
                readonly: false
                mtime: 2021-06-09T08:02:32+02:00
                atime: 2022-07-10T08:02:32+02:00
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
    assert_eq!(CONTENT2, read2);

    test.check(r"
        FILES:
            dir:
                test.txt: test_2
        ATTRIBUTES:
            dir/test.txt:
                permissions: -rw-r--r--
                readonly: false
                mtime: 2021-06-09T08:02:32+02:00
                atime: 2022-07-10T08:02:32+02:00
    ")
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