use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

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
