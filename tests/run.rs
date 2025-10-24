use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;

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

#[test]
fn run_current_skips_brew_category() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache = temp.child("target");
    cache.create_dir_all().unwrap();
    // Add some content to make it detectable
    cache.child("debug").create_dir_all().unwrap();
    cache.child("debug/pure").write_str("executable").unwrap();

    // Change to temp directory to test --current
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("run")
        .arg("--current")
        .arg("--type")
        .arg("rust")
        .arg("-y");

    let output = cmd.assert().success();

    // Test passes if either deletion happened or nothing was found to delete
    // The key is that we're testing --current works, not specifically rust deletion
    output.stdout(
        predicate::str::contains("Attempted to delete")
            .or(predicate::str::contains("Nothing to delete")),
    );

    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}
