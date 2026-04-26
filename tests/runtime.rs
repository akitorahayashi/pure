#[allow(dead_code, unused_imports)]
mod harness;

use harness::TestContext;
use predicates::prelude::*;

#[test]
fn binary_exists_and_runs() {
    let ctx = TestContext::new();

    ctx.cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_subcommand_fails() {
    let ctx = TestContext::new();

    ctx.cli().arg("unknown-command").assert().failure().stderr(predicate::str::is_empty().not());
}
