use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn command() -> Command {
    Command::cargo_bin("pure").expect("binary exists")
}

#[test]
fn config_add_exclude_prevents_scan_hits() {
    let temp = assert_fs::TempDir::new().unwrap();
    let home = temp.child("home");
    home.create_dir_all().unwrap();
    let config_root = temp.child("xdg-config");
    config_root.create_dir_all().unwrap();

    let cache = home.child("project/node_modules");
    cache.create_dir_all().unwrap();
    cache.child("a.js").write_str("cache").unwrap();

    // Add exclude entry
    let mut config_cmd = command();
    config_cmd
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", config_root.path())
        .arg("config")
        .arg("--add-exclude")
        .arg("~/project/node_modules");
    config_cmd.assert().success();

    let config_path = config_root.child("pure/config.toml");
    let contents = fs::read_to_string(config_path.path()).unwrap();
    assert!(contents.contains("project/node_modules"));

    // Scan should now ignore the directory
    let mut scan_cmd = command();
    scan_cmd
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", config_root.path())
        .arg("scan")
        .arg("--type")
        .arg("dev")
        .arg("--verbose")
        .arg(home.path());

    scan_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Scan results"))
        .stdout(predicate::str::contains("Total reclaimable: 0 B"))
        .stdout(predicate::str::contains("dev"));

    // Path option prints location
    let mut path_cmd = command();
    path_cmd
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", config_root.path())
        .arg("config")
        .arg("--path");
    path_cmd.assert().success().stdout(predicate::str::contains("pure/config.toml"));
}
