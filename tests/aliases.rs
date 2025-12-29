use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn command() -> Command {
    Command::cargo_bin("pure").expect("binary exists")
}

#[test]
fn alias_sc_works_like_scan() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project = temp.child("project/__pycache__");
    project.create_dir_all().unwrap();
    project.child("foo.pyc").write_str("cache").unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("sc")
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
fn alias_rn_works_like_run() {
    // Just testing it accepts the command, using --help to avoid actual execution side effects complexity
    // and because without arguments run might prompt or error depending on implementation
    let mut cmd = command();
    cmd.arg("rn").arg("--help");

    cmd.assert().success().stdout(predicate::str::contains("Delete files discovered by a scan"));
}

#[test]
fn alias_cfg_works_like_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = command();
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.child("config").path())
        .arg("cfg")
        .arg("--path");

    cmd.assert().success().stdout(predicate::str::contains("config.toml"));
}
