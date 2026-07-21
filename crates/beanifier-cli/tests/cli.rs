//! End-to-end tests that spawn the real `beanify` binary.

use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn beanify() -> Command {
    Command::new(env!("CARGO_BIN_EXE_beanify"))
}

#[test]
fn beanifies_a_file_to_stdout() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("note.txt");
    fs::write(&file, "hello there").unwrap();

    let out = beanify().arg(&file).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("hello"));
    assert!(!stdout.is_empty());
}

#[test]
fn recurses_a_directory_in_place() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), "one two").unwrap();
    fs::write(dir.path().join("sub/b.txt"), "three four").unwrap();

    let status = beanify()
        .arg("--in-place")
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    assert!(!fs::read_to_string(dir.path().join("a.txt"))
        .unwrap()
        .contains("one"));
    assert!(!fs::read_to_string(dir.path().join("sub/b.txt"))
        .unwrap()
        .contains("three"));
}

#[test]
fn is_deterministic_across_invocations() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("d.txt");
    fs::write(&file, "the quick brown fox").unwrap();

    let a = beanify()
        .arg("--seed")
        .arg("99")
        .arg(&file)
        .output()
        .unwrap();
    let b = beanify()
        .arg("--seed")
        .arg("99")
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn missing_path_fails() {
    let status = beanify().arg("/no/such/path/here").status().unwrap();
    assert!(!status.success());
}
