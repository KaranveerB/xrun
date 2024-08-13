mod command_parser;

use std::{
    env,
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{Command, Stdio},
};

use command_parser::{get_command, get_command_help, CommandParseError, HelpPair};

#[derive(PartialEq)]
enum Action {
    Exec,
    Help,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let executable_name = args.first().unwrap();
    // TODO: This is not robust enough for flags that also take an arg
    let (options, command): (Vec<_>, Vec<_>) = args[1..]
        .iter()
        .map(|s| s.as_str())
        .partition(|&s| s.starts_with('-'));

    let mut action = Action::Exec;
    for option in options {
        match option {
            "--help" | "-h" => action = Action::Help,
            _ => {
                eprintln!("Unknown flag: {}", option);
                std::process::exit(1)
            }
        }
    }

    if command.is_empty() && action != Action::Help {
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
    match action {
        Action::Exec => command_runner(path, command).or_disp_and_die(),
        Action::Help => help_runner(executable_name, path, command).or_disp_and_die(),
    }
    unreachable!()
}

fn command_runner(path: &Path, command: Vec<&str>) -> Result<(), CommandParseError> {
    let exec_command = get_command(path, &command)?;
    let mut proc = Command::new("sh")
        .arg("-c")
        .arg(exec_command)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let status = proc.wait()?;
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

fn help_runner(
    executable_name: &str,
    path: &Path,
    command: Vec<&str>,
) -> Result<(), CommandParseError> {
    let help_pairs = get_command_help(path, &command)?;
    print!("Usage: {} ", executable_name);
    for command in command {
        print!("{} ", command);
    }
    println!("[command]\n\ncommands:");
    for HelpPair(cmd, desc) in help_pairs {
        match (cmd, desc) {
            (Some(cmd), Some(desc)) => println!("  {}: {}", cmd, desc),
            (Some(cmd), None) => println!("  {}", cmd),
            (None, Some(desc)) => println!("{}", desc),
            (None, None) => {}
        }
    }
    std::process::exit(0)
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
