// TODO: Make tests that test output be more robust somehow.

use assert_cmd::Command;
use std::fs;

use rstest::{fixture, rstest};
use tempfile::TempDir;

const TOML_COMMAND_DATA: &[u8] = r#"
    [s]
    c1 = { command = "echo c1 ran", desc = "run c1"}
    c2 = { command = "echo c2 ran", desc = "run c2"}
"#
.as_bytes();

// Struct to keep tmp_dir in memory while passing data from fixtures.
struct TestSetup {
    _tmp_dir: TempDir,
    cmd: assert_cmd::Command,
}

// TODO: This works fine when just using `TOML_COMMAND_DATA` and all other configs would need their
// own fixture or repeated code. This is probably fine, but try to find a better way to do this.
#[fixture]
fn basic_cmd() -> TestSetup {
    let tmp_dir = TempDir::new().unwrap();
    let _ = fs::create_dir(tmp_dir.path().join("srun"));
    let _ = fs::write(tmp_dir.path().join("srun/command.toml"), TOML_COMMAND_DATA);
    let mut cmd = Command::cargo_bin("srun").unwrap();
    cmd.env("XDG_CONFIG_HOME", tmp_dir.path());

    TestSetup {
        _tmp_dir: tmp_dir,
        cmd,
    }
}

#[rstest]
fn test_exec_success(mut basic_cmd: TestSetup) {
    let assert = basic_cmd.cmd.args("s c1".split_whitespace()).assert();
    assert.success().stdout("c1 ran\n");
}

#[rstest]
fn test_exec_subcommand_dne(mut basic_cmd: TestSetup) {
    let assert = basic_cmd.cmd.args("dne c1".split_whitespace()).assert();
    assert.failure().code(1).stdout("").stderr("Error: Command 'dne c1' not found\n");
}

#[rstest]
fn test_exec_command_dne(mut basic_cmd: TestSetup) {
    let assert = basic_cmd.cmd.args("s dne".split_whitespace()).assert();
    assert.failure().code(1).stdout("").stderr("Error: Command 'dne' not found\n");
}
