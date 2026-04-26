use crate::harness::TestContext;
use predicates::prelude::*;

#[test]
fn run_type_nodejs_yes_deletes_directories() {
    let ctx = TestContext::new();
    let cache = ctx.write_home_file("workspace/node_modules/index.js", "console.log('cache');");
    let cache_dir = cache.parent().expect("cache file has parent").to_path_buf();

    ctx.cli()
        .arg("run")
        .arg("--type")
        .arg("nodejs")
        .arg("-y")
        .arg(ctx.home())
        .assert()
        .success()
        .stdout(predicate::str::contains("Attempted to delete"));

    assert!(!cache_dir.exists(), "cache directory should be deleted");
}

#[test]
fn run_interactive_accepts_selection() {
    let ctx = TestContext::new();
    let cache = ctx.write_home_file("workspace/__pycache__/foo.pyc", "cache");
    let cache_dir = cache.parent().expect("cache file has parent").to_path_buf();

    ctx.cli()
        .arg("run")
        .arg(ctx.home())
        .write_stdin("python\ny\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deletion plan"))
        .stdout(predicate::str::contains("Attempted to delete"));

    assert!(!cache_dir.exists(), "cache directory should be deleted");
}
