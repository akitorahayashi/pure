use crate::harness::TestContext;
use predicates::prelude::*;

#[test]
fn alias_sc_works_like_scan() {
    let ctx = TestContext::new();
    ctx.write_home_file("project/__pycache__/foo.pyc", "cache");

    ctx.cli()
        .arg("sc")
        .arg("--type")
        .arg("python")
        .arg("--verbose")
        .arg(ctx.home())
        .assert()
        .success()
        .stdout(predicate::str::contains("Scan results"))
        .stdout(predicate::str::contains("Python"))
        .stdout(predicate::str::contains("~/project/__pycache__"));
}

#[test]
fn alias_rn_works_like_run() {
    let ctx = TestContext::new();

    ctx.cli()
        .arg("rn")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Delete files discovered by a scan"));
}
