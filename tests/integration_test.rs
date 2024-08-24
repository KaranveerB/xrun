// TODO: Make tests that test output be more robust somehow.

use assert_cmd::Command;
use std::fs;

use rstest::{fixture, rstest};
use tempfile::TempDir;

const BASIC_TOML_COMMAND_DATA: &[u8] = r#"
    [s]
    desc = "s desc"
    c1 = { command = "echo c1 ran", desc = "c1 desc" }
    c2 = { command = "echo c2 ran" }
"#
.as_bytes();

// Struct to keep tmp_dir in memory while passing data from fixtures.
struct TestSetup {
    _tmp_dir: TempDir,
    cmd: assert_cmd::Command,
}

fn create_test_setup(config: &[u8]) -> TestSetup {
    let tmp_dir = TempDir::new().unwrap();
    let _ = fs::create_dir(tmp_dir.path().join("srun"));
    let _ = fs::write(tmp_dir.path().join("srun/command.toml"), config);
    let mut cmd = Command::cargo_bin("srun").unwrap();
    cmd.env("XDG_CONFIG_HOME", tmp_dir.path());

    TestSetup {
        _tmp_dir: tmp_dir,
        cmd,
    }
}

// TODO: This works fine when just using `TOML_COMMAND_DATA` and all other configs would need their
// own fixture or repeated code. This is probably fine, but try to find a better way to do this.
#[fixture]
fn basic_cmd() -> TestSetup {
    create_test_setup(BASIC_TOML_COMMAND_DATA)
}

fn test_cmd(mut test_setup: TestSetup, arg_str: &str, stdout: &str, stderr: &str, ret: i32) {
    let assert = test_setup.cmd.args(arg_str.split_whitespace()).assert();
    assert
        .code(ret)
        .stdout(stdout.to_owned())
        .stderr(stderr.to_owned());
}

#[rstest]
fn test_exec_success(basic_cmd: TestSetup) {
    test_cmd(basic_cmd, "s c1", "c1 ran\n", "", 0);
}

#[rstest]
fn test_exec_subcommand_dne(basic_cmd: TestSetup) {
    let stderr = "Error: Command 'dne c1' not found\n";
    test_cmd(basic_cmd, "dne c1", "", stderr, 1);
}

#[rstest]
fn test_exec_command_dne(basic_cmd: TestSetup) {
    let stderr = "Error: Command 'dne' not found\n";
    test_cmd(basic_cmd, "s dne", "", stderr, 1);
}

#[test]
fn test_exec_passthrough_stderr() {
    let toml_command_data = r#"c = { command = ">&2 echo 'foo'" }"#.as_bytes();
    let stderr = "foo\n";
    let test_setup = create_test_setup(toml_command_data);
    test_cmd(test_setup, "c", "", stderr, 0);
}

#[test]
fn test_exec_passthrough_ret_code() {
    let toml_command_data = r#"c = { command = "exit 42" }"#.as_bytes();
    let test_setup = create_test_setup(toml_command_data);
    test_cmd(test_setup, "c", "", "", 42);
}

#[test]
fn test_exec_passthrough_signal() {
    let toml_command_data = r#"c = { command = "kill -s TERM $$" }"#.as_bytes();
    let test_setup = create_test_setup(toml_command_data);
    test_cmd(test_setup, "c", "", "", 15 + 128);
}

#[test]
fn test_exec_passthrough_stdin() {
    let toml_command_data = r#"c = { command = "read line; echo $line" }"#.as_bytes();
    let mut test_setup = create_test_setup(toml_command_data);
    let stdout = "foo\n";
    let assert = test_setup.cmd.arg("c").write_stdin("foo").assert();
    let _ = assert.success().stdout(stdout).stderr("");
}

/// Test help for a subcommand with a description and child commands.
#[rstest]
fn test_help_subcommand(basic_cmd: TestSetup) {
    let stdout = concat!(
        "usage: srun s [command]\n",
        "s desc\n",
        "\n",
        "commands:\n",
        "    c1: c1 desc\n",
        "    c2\n",
    );
    test_cmd(basic_cmd, "s --help", stdout, "", 0);
}

/// Test help for a subcommand with a description and child commands.
#[test]
fn test_help_subcommand_no_desc() {
    let toml_command_data: &[u8] = r#"
        [s]
        c1 = { command = "echo c1 ran", desc = "c1 desc" }
        c2 = { command = "echo c2 ran" }
    "#
    .as_bytes();
    let stdout = concat!(
        "usage: srun s [command]\n",
        "commands:\n",
        "    c1: c1 desc\n",
        "    c2\n",
    );
    let test_setup = create_test_setup(toml_command_data);
    test_cmd(test_setup, "s --help", stdout, "", 0);
}

#[rstest]
fn test_help_command(basic_cmd: TestSetup) {
    let stdout = concat!("usage: srun s c1\n", "c1 desc\n");
    test_cmd(basic_cmd, "s c1 --help", stdout, "", 0);
}

#[rstest]
fn test_help_command_no_desc(basic_cmd: TestSetup) {
    let stdout = "usage: srun s c2\n";
    test_cmd(basic_cmd, "s c2 --help", stdout, "", 0);
}

#[rstest]
fn test_help_subcommand_dne(basic_cmd: TestSetup) {
    let stderr = "Error: Command 'dne dne' not found\n";
    test_cmd(basic_cmd, "dne dne --help", "", stderr, 1);
}

#[rstest]
fn test_help_command_dne(basic_cmd: TestSetup) {
    let stderr = "Error: Command 'dne' not found\n";
    test_cmd(basic_cmd, "s dne --help", "", stderr, 1);
}
