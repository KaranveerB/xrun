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

const PROG_NAME: &str = "srun";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    // TODO: This is not robust enough for flags that also take an arg
    let (options, command): (Vec<_>, Vec<_>) = args
        .iter()
        .map(|s| s.as_str())
        .partition(|&s| s.starts_with('-'));

    let mut action = Action::Exec;
    let mut passthrough = false;
    for option in options {
        match option {
            "--help" | "-h" => action = Action::Help,
            "--passthrough" | "-p" => passthrough = true,
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
        Action::Exec => command_runner(path, &command, passthrough).or_disp_and_die(),
        Action::Help => help_runner(path, &command).or_disp_and_die(),
    }
    unreachable!()
}

fn command_runner(
    path: &Path,
    command: &[&str],
    passthrough: bool,
) -> Result<(), CommandParseError> {
    let exec_command = get_command(path, command)?;
    if passthrough {
        println!("{}", exec_command);
        // Arbitrary exit code to indicate a shell command was returned.
        std::process::exit(125);
    } else {
        let shell = env::var("SHELL").unwrap_or("sh".to_string());

        let mut command = &mut Command::new(&shell);

        if shell.ends_with("bash") || shell.ends_with("zsh") || shell.ends_with("fish") {
            // Many programs use isatty for things like whether to add colours. Make sure we pass
            // interactive is isatty passes and we get as close to real shell aliases as possible.
            command = command.arg("-i");
        };

        command = command
            .arg("-c") // Assume whatever shell is used supports -c
            .arg(exec_command)
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut proc = command.spawn()?;
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
}

fn help_runner(path: &Path, command: &[&str]) -> Result<(), CommandParseError> {
    let help_pairs = get_command_help(path, command)?;
    print!("usage: {}", PROG_NAME);
    for command in command {
        print!(" {}", command);
    }
    if help_pairs.len() > 1 {
        print!(" [command]");
    }
    println!();
    let base_command = help_pairs.iter().find(|e| e.0.is_none());
    if let Some(help_pair) = base_command {
        if let Some(desc) = &help_pair.1 {
            println!("{}", desc);
            if help_pairs.len() > 1 {
                println!();
            }
        }
    }
    if help_pairs.len() > 1 {
        println!("commands:");
        for HelpPair(cmd, desc) in help_pairs {
            match (cmd, desc) {
                (Some(cmd), Some(desc)) => println!("    {}: {}", cmd, desc),
                (Some(cmd), None) => println!("    {}", cmd),
                (None, _) => {} // already shown
            }
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
