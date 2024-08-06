mod command_parser;

use std::process::Command;

use command_parser::{get_command, CommandParseError};

fn main() -> Result<(), CommandParseError> {
    let path = "tests/res/basic.toml";
    let command = "subcommand1 command1";
    let exec_command = get_command(path, command)?;
    let command_output = Command::new("sh").arg("-c").arg(exec_command).output()?;
    if command_output.status.success() {
        let stdout = String::from_utf8_lossy(&command_output.stdout);
        print!("{}", stdout);
    }
    Ok(())
}
