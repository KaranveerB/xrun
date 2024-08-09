mod command_parser;

use std::{env, path::Path, process::Command};

use command_parser::{get_command, CommandParseError};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let command: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    if command.is_empty() {
        eprintln!("No command provided");
        std::process::exit(1);
    }
    if let Err(err) = command_runner(command) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
    std::process::exit(0);
}

fn command_runner(command: Vec<&str>) -> Result<(), CommandParseError> {
    let path = Path::new("tests/res/basic.toml");
    let exec_command = get_command(path, command)?;
    let command_output = Command::new("sh").arg("-c").arg(exec_command).output()?;
    if command_output.status.success() {
        let stdout = String::from_utf8_lossy(&command_output.stdout);
        print!("{}", stdout);
    }
    Ok(())
}
