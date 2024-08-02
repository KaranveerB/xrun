use std::{fs, io, process::Command};
use toml::{self, Table, Value};

#[derive(Debug)]
enum CommandReadError {
    IoError(io::Error),
    TomlDeError(toml::de::Error),
    /// An error for when a command is not found in the config files.
    /// The `String` parameter is the component of the command that is not found.
    CommandNotFoundError(String),
    // An error for when an entry for the command/subcommand is found, but the
    // contents are not valid for command parsing.
    // The `String` parameters represents the invalid command portion and reason respectively.
    CommandContentInvalid(String, String),
}

impl std::fmt::Display for CommandReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandReadError::IoError(err) => write!(f, "{}", err),
            CommandReadError::TomlDeError(err) => write!(f, "TOML parse error: {}", err),
            CommandReadError::CommandNotFoundError(err) => {
                write!(f, "Command not found - Missing component: {}", err)
            }
            CommandReadError::CommandContentInvalid(command, reason) => {
                write!(f, "Command content invalid - {}: {}", command, reason)
            }
        }
    }
}

impl std::error::Error for CommandReadError {}

impl From<io::Error> for CommandReadError {
    fn from(err: io::Error) -> Self {
        CommandReadError::IoError(err)
    }
}

impl From<toml::de::Error> for CommandReadError {
    fn from(err: toml::de::Error) -> Self {
        CommandReadError::TomlDeError(err)
    }
}

fn read_toml_to_map(path: &str) -> Result<toml::map::Map<String, toml::Value>, CommandReadError> {
    let toml_str = fs::read_to_string(path)?;
    let toml_data: Table = toml::from_str(&toml_str)?;
    Ok(toml_data)
}

fn get_command(path: &str, command: &str) -> Result<String, CommandReadError> {
    let mut toml_data = read_toml_to_map(path)?;
    let mut command_not_found = false;
    let mut error_string: String = Default::default();
    for token in command.split_whitespace() {
        if !command_not_found {
            match toml_data.get(token) {
                Some(Value::Table(next_table)) => {
                    toml_data = next_table.to_owned();
                }
                Some(_) => {
                    return Err(CommandReadError::CommandContentInvalid(
                        token.to_owned(),
                        "".to_string(),
                    ));
                }
                None => {
                    command_not_found = true;
                    error_string += token;
                }
            }
        } else {
            error_string += " ";
            error_string += token;
        }
    }
    if command_not_found {
        Err(CommandReadError::CommandNotFoundError(error_string))
    } else {
        match toml_data.get("command") {
            Some(exec_cmd) => match exec_cmd.as_str() {
                Some(exec_cmd) => Ok(exec_cmd.to_string()),
                None => Err(CommandReadError::CommandContentInvalid(
                    command.to_owned(),
                    "`command` value is not a string".to_string(),
                )),
            },
            None => Err(CommandReadError::CommandContentInvalid(
                command.to_owned(),
                "`command` key/value not found".to_string(),
            )),
        }
    }
}

fn main() -> Result<(), CommandReadError> {
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
