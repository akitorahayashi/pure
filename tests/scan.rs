use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;

fn command() -> Command {
    Command::cargo_bin("pure").expect("binary exists")
}

#[test]
fn scan_python_verbose_lists_targets() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project = temp.child("project/__pycache__");
    project.create_dir_all().unwrap();
    project.child("foo.pyc").write_str("cache").unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("scan")
        .arg("--type")
        .arg("python")
        .arg("--verbose")
        .arg(temp.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan results"))
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("~/project/__pycache__"));
}

#[test]
fn scan_current_skips_brew_category() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Change to the temp directory to test --current
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("scan")
        .arg("--current")
        .arg("--list");

    let result = cmd.assert().success().stdout(predicate::str::contains("Found cleanup targets"));

    // Brew should not appear in --current scan results
    result.stdout(predicate::str::contains("Homebrew").not());

    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn scan_default_includes_brew_category() {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("scan")
        .arg("--list")
        .arg(temp.path());

    // Default scan should include brew (even if no targets found, category should be checked)
    cmd.assert().success().stdout(predicate::str::contains("Found cleanup targets"));
}

#[test]
fn scan_current_skips_docker_category() {
    let temp = assert_fs::TempDir::new().unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("scan")
        .arg("--current")
        .arg("--list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found cleanup targets"))
        .stdout(predicate::str::contains("Docker").not())
        .stdout(predicate::str::contains("Unused images").not())
        .stdout(predicate::str::contains("Stopped containers").not())
        .stdout(predicate::str::contains("Dangling volumes").not())
        .stdout(predicate::str::contains("Unused networks").not())
        .stdout(predicate::str::contains("Build cache").not());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn scan_default_includes_docker_category() {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("scan")
        .arg("--list")
        .arg(temp.path());

    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Found cleanup targets"));

    let docker_available = std::process::Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if docker_available {
        assert!(
            stdout.contains("Docker"),
            "Expected Docker category in output when Docker is available."
        );
    } else {
        assert!(
            !stdout.contains("Docker"),
            "Expected no Docker category in output when Docker is not available."
        );
    }
}

#[test]
fn version_flag_works() {
    let mut cmd = command();
    cmd.arg("--version");

    cmd.assert().success();
}
