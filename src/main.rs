mod command_parser;

use std::{
    env,
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

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
    let status = command_runner(path, command).or_disp_and_die();
    let exit_code = match status.code() {
        Some(code) => code,
        None => match status.signal() {
            Some(signal) => 128 + signal,
            None => {
                panic!("Unknown exit status {:?}", status);
            }
        },
    };
    std::process::exit(exit_code);
}

fn command_runner(path: &Path, command: Vec<&str>) -> Result<ExitStatus, CommandParseError> {
    let exec_command = get_command(path, command)?;
    let mut proc = Command::new("sh")
        .arg("-c")
        .arg(exec_command)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let status = proc.wait()?;
    Ok(status)
}

trait OrDispAndDie<T, F> {
    fn or_disp_and_die(self) -> T
    where
        F: std::fmt::Display;
}

impl<T, F> OrDispAndDie<T, F> for Result<T, F>
where
    F: std::fmt::Display,
{
    fn or_disp_and_die(self) -> T {
        self.unwrap_or_else(|err| {
            eprintln!("Error: {}", err); // Print the error message to stderr
            std::process::exit(1); // Exit the program with status code 1
        })
    }
}
