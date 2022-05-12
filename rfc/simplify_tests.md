# Simplify tests
## Summary
Using declarative approach for create and check folders content.  For declaration description using yaml string data

### Example YAML data:
```yaml
Test name for folder: "FOLDER"
	Test name for file: "FILE"
	Another file: 
		- type: "FILE"
		- content: "blah blah blah"
	Another folder: "FOLDER"
```

## Motivation
I think every to agree with me, that now we have many code noise in tests. 
### Example:
```rust 
    let mut test_file = PathBuf::from(TEST_FOLDER);
    test_file.push("it_read_and_write_work");
    test_file.push("test.txt");
    fs_extra::dir::create_all(test_file.parent().unwrap(), true).unwrap();
    let content1 = "test_1";
    let content2 = "test_2";
    write_all(&test_file, &content1).unwrap();
    assert!(test_file.exists());
    let read1 = read_to_string(&test_file).unwrap();
    assert_eq!(content1, read1);
    write_all(&test_file, &content2).unwrap();
    let read2 = read_to_string(&test_file).unwrap();
    assert_eq!(content2, read2);
```

In new way this code can be like:

```rust 
let t = TestFS::new("it_read_and_write_work", r#"	
			test.txt: 
				type: "FILE"
				content: "test_1""#;
let test_file = t.get_path("test.txt")
let data = read_to_string(&test_file).unwrap();
assert_eq!("test_1", data);
write_all(&test_file, "test_2").unwrap();
data = read_to_string(&test_file).unwrap();
assert_eq!("test_2", data);
```

I think we can change it and make tests more readable, and increase speed to write new tests using new approach. 

### Another Example:
```rust
let mut test_file = PathBuf::from(TEST_FOLDER);
test_file.push("it_copy_work");
let mut test_file_out = test_file.clone();
test_file.push("test.txt");
test_file_out.push("out");
test_file_out.push("test.txt");
fs_extra::dir::create_all(&test_file.parent().unwrap(), true).unwrap();
fs_extra::dir::create_all(&test_file_out.parent().unwrap(), true).unwrap();

write_all(&test_file, "test_data").unwrap();
assert!(test_file.exists());
assert!(!test_file_out.exists());
let options = CopyOptions::new();
copy(&test_file, &test_file_out, &options).unwrap();
assert!(test_file.exists());
assert!(test_file_out.exists());
assert_eq!(test_file.file_name(), test_file_out.file_name());
assert!(files_eq(test_file, test_file_out).unwrap());
```
RFC version:
```rust
let t = TestFS::new("it_copy_work", r#"	
    test.txt: 
      type: "FILE"
      content: "test_data"
    out: "FOLDER""#);
let options = CopyOptions::new();
let test_file = t.get_path("test.txt")
let test_file_out = t.get_path("out/test.txt")
copy(&test_file, &test_file_out, &options).unwrap();
t.check(r#"	
  test.txt: 
    type: "FILE"
    content: "test_data"
  out: "FOLDER"
    test.txt: 
      type: "FILE"
      content: "test_data""#);
```



