use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn command() -> Command {
    Command::cargo_bin("pure").expect("binary exists")
}

#[test]
fn config_path_shows_location() {
    let temp = assert_fs::TempDir::new().unwrap();
    let home = temp.child("home");
    home.create_dir_all().unwrap();
    let config_root = temp.child("xdg-config");
    config_root.create_dir_all().unwrap();

    // Path option prints location
    let mut path_cmd = command();
    path_cmd
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", config_root.path())
        .arg("config")
        .arg("--path");
    path_cmd.assert().success().stdout(predicate::str::contains("pure/config.toml"));
}
