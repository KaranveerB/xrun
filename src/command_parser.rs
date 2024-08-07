use std::{fs, io};
use toml::{self, Table, Value};

#[derive(Debug)]
/// Errors when parsing and searching for commands from the config.
pub(crate) enum CommandParseError {
    /// Wrapper for `io::Error`
    IoError(io::Error),
    /// Wrapper for `toml::de::Error`
    TomlDeError(toml::de::Error),
    /// An error for when a command is not found in the config files.
    ///
    /// * `String` - The component of the command that is not found.
    CommandNotFoundError(String),
    // An error for when an entry for the command/subcommand is found, but the
    // contents are not valid for command parsing.
    //
    // * `String` The invalid component of the command.
    // * `String` The reason the component of the command is invalid.
    CommandContentInvalid(String, String),
}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandParseError::IoError(err) => write!(f, "{}", err),
            CommandParseError::TomlDeError(err) => write!(f, "TOML parse error: {}", err),
            CommandParseError::CommandNotFoundError(err) => {
                write!(f, "Command not found - Missing component: {}", err)
            }
            CommandParseError::CommandContentInvalid(command, reason) => {
                write!(f, "Command content invalid - {}: {}", command, reason)
            }
        }
    }
}

impl std::error::Error for CommandParseError {}

impl From<io::Error> for CommandParseError {
    fn from(err: io::Error) -> Self {
        CommandParseError::IoError(err)
    }
}

impl From<toml::de::Error> for CommandParseError {
    fn from(err: toml::de::Error) -> Self {
        CommandParseError::TomlDeError(err)
    }
}

/// Creates of a table of the `toml_str` toml data.
///
/// * `toml_str` - The toml to parse.
///
/// returns - The loaded key-value table or CommandParseError::TomlDeError.
pub(crate) fn toml_to_map(
    toml_str: &str,
) -> Result<toml::map::Map<String, toml::Value>, CommandParseError> {
    let toml_data: Table = toml::from_str(&toml_str)?;
    Ok(toml_data)
}

/// Parses a .toml file and extracts the action of a specified command.
///
/// * `path` - The path to the .toml file of the base command file.
/// * `command` - The specified command to retrieve the action of.
///
/// returns - The command action if the command is present, or the error that occured while retrieving the command action.
pub(crate) fn get_command(path: &str, command: &str) -> Result<String, CommandParseError> {
    let toml_str = &fs::read_to_string(path)?;
    let mut toml_data = toml_to_map(toml_str)?;
    let mut command_not_found = false;
    let mut error_string: String = Default::default();
    for token in command.split_whitespace() {
        if !command_not_found {
            match toml_data.get(token) {
                Some(Value::Table(next_table)) => {
                    toml_data = next_table.to_owned();
                }
                Some(_) => {
                    return Err(CommandParseError::CommandContentInvalid(
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
        Err(CommandParseError::CommandNotFoundError(error_string))
    } else {
        match toml_data.get("command") {
            Some(exec_cmd) => match exec_cmd.as_str() {
                Some(exec_cmd) => Ok(exec_cmd.to_string()),
                None => Err(CommandParseError::CommandContentInvalid(
                    command.to_owned(),
                    "`command` value is not a string".to_string(),
                )),
            },
            None => Err(CommandParseError::CommandContentInvalid(
                command.to_owned(),
                "`command` key/value not found".to_string(),
            )),
        }
    }
}
