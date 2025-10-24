use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn command() -> Command {
    Command::cargo_bin("pure").expect("binary exists")
}

#[test]
fn run_type_dev_yes_deletes_directories() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache = temp.child("workspace/node_modules");
    cache.create_dir_all().unwrap();
    cache.child("index.js").write_str("console.log('cache');").unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("run")
        .arg("--type")
        .arg("nodejs")
        .arg("-y")
        .arg(temp.path());

    cmd.assert().success().stdout(predicate::str::contains("Attempted to delete"));

    cache.assert(predicates::path::missing());
}

#[test]
fn run_interactive_accepts_selection() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache = temp.child("workspace/__pycache__");
    cache.create_dir_all().unwrap();
    cache.child("foo.pyc").write_str("cache").unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("run")
        .arg(temp.path())
        .write_stdin("python\ny\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deletion plan"))
        .stdout(predicate::str::contains("Attempted to delete"));

    cache.assert(predicates::path::missing());
}
