mod command_parser;

use std::{env, path::Path, process::Command};

use command_parser::{get_command, CommandParseError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    let command: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    if command.is_empty() {
        eprintln!("Error: No command provided");
        std::process::exit(1);
    }

    let xdg_dirs = xdg::BaseDirectories::with_prefix("srun")?;
    let path: std::path::PathBuf = xdg_dirs
        .find_config_file("command.toml")
        .unwrap_or_else(|| {
            eprintln!("Error: command.toml does not exist in config directory");
            std::process::exit(1);
        });
    let path: &Path = path.as_path();
    command_runner(path, command)?;
    Ok(())
}

fn command_runner(path: &Path, command: Vec<&str>) -> Result<(), CommandParseError> {
    let exec_command = get_command(path, command)?;
    let command_output = Command::new("sh").arg("-c").arg(exec_command).output()?;
    if command_output.status.success() {
        let stdout = String::from_utf8_lossy(&command_output.stdout);
        print!("{}", stdout);
    }
    Ok(())
}
