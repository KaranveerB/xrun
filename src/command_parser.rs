use std::{fs, io, path::Path};
use toml::{self, Table, Value};

/// Reason why a toml key/value is considered contextually invalid during command parsing.
#[derive(Debug)]
pub(crate) enum InvalidContentReason {
    /// Expected a toml string but got something else.
    ///
    /// * `String` - The key which is not a table.
    /// * `Value` - The actual value received.
    NotTomlString(String, Value),
    /// Expected a toml table but got something else.
    ///
    /// * `String` - The key which is not a table.
    /// * `Value` - The actual value received.
    NotTomlTable(String, Value),
    /// A key, such as 'command' is not present when it was expected to be.
    ///
    /// * `String` - The expected key that is not present.
    MissingKey(String),
}

/// Gets a string representation of the type (actually enum value) of the Value.
fn value_as_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Float(_) => "Float",
        Value::Boolean(_) => "Boolean",
        Value::Datetime(_) => "Datetime",
        Value::Array(_) => "Array",
        Value::Table(_) => "Table",
    }
}

impl std::error::Error for InvalidContentReason {}

impl std::fmt::Display for InvalidContentReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidContentReason::NotTomlString(component, value) => write!(
                f,
                "Expected key '{}' to be String but got {}",
                component,
                value_as_name(value)
            ),
            InvalidContentReason::NotTomlTable(component, value) => write!(
                f,
                "Expected key '{}' to be Table but got {}",
                component,
                value_as_name(value)
            ),
            InvalidContentReason::MissingKey(key) => {
                write!(f, "Expected key '{}' but it is not present", key)
            }
        }
    }
}

/// Errors when parsing and searching for commands from the config.
#[derive(Debug)]
pub(crate) enum CommandParseError {
    /// Wrapper for `io::Error`
    IoError(io::Error),
    /// Wrapper for `toml::de::Error`
    TomlDeError(toml::de::Error),
    /// An error for when a command is not found in the config files.
    ///
    /// * `String` - The component of the command that is not found.
    CommandNotFoundError(String),
    ///
    /// An error for when an entry is present, but there is no valid execution.
    CommandContentInvalid(InvalidContentReason),
}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandParseError::IoError(err) => write!(f, "{}", err),
            CommandParseError::TomlDeError(err) => write!(f, "TOML parse error - {}", err),
            CommandParseError::CommandNotFoundError(err) => {
                write!(f, "Command `{}` not found)", err)
            }
            CommandParseError::CommandContentInvalid(err) => {
                write!(f, "Command content invalid - {}", err)
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

impl From<InvalidContentReason> for CommandParseError {
    fn from(err: InvalidContentReason) -> Self {
        CommandParseError::CommandContentInvalid(err)
    }
}

/// Creates of a table of the `toml_str` toml data.
///
/// * `toml_str` - The toml to parse.
///
/// returns - The loaded key-value table or `CommandParseError::TomlDeError`.
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
/// returns - The command action if the command is present, or the error that occurred while retrieving the command action.
pub(crate) fn get_command(path: &Path, command: Vec<&str>) -> Result<String, CommandParseError> {
    let toml_str = &fs::read_to_string(path)?;
    let mut toml_data = toml_to_map(toml_str)?;
    let mut command_not_found = false;
    let mut error_string: String = Default::default();
    for token in command {
        if !command_not_found {
            match toml_data.get(token) {
                Some(Value::Table(next_table)) => {
                    toml_data = next_table.to_owned();
                }
                Some(value) => {
                    return Err(CommandParseError::CommandContentInvalid(
                        InvalidContentReason::NotTomlTable(token.to_owned(), value.to_owned()),
                    ));
                }
                None => {
                    command_not_found = true;
                    error_string += &token;
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
                    InvalidContentReason::NotTomlString("command".to_string(), exec_cmd.to_owned()),
                )),
            },
            None => Err(CommandParseError::CommandContentInvalid(
                InvalidContentReason::MissingKey("command".to_string()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use io::Write;

    use tempfile::NamedTempFile;
    use test_case::test_case;

    use super::*;

    const TOML_COMMAND_DATA: &[u8] = r#"
            [foo]
            bar = { command = "bar exec" }
            qux = "quux"
            command = { }
            [baz]
        "#
    .as_bytes();

    #[test]
    fn test_toml_to_map_invalid() {
        let toml_str = "invalid toml";
        let result = toml_to_map(toml_str);
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandParseError::TomlDeError(_) => {}
            err => panic!("Expected CommandParseError::TomlDeError, got {:?}", err),
        }
    }

    #[test]
    fn test_get_command_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        temp_file
            .reopen()
            .unwrap()
            .write_all(TOML_COMMAND_DATA)
            .unwrap();
        let result = get_command(temp_file.path(), "foo bar".split_whitespace().collect());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bar exec")
    }

    #[test_case("bar",  "bar"  ; "skipped subcommand")]
    #[test_case("foo baz",  "baz"  ; "bad command child of valid subcommand")]
    #[test_case("foo bar baz",  "baz"  ; "bad command child of valid command")]
    #[test_case("foo bar baz qux quux",  "baz qux quux"  ; "bad command with multiple invalid component")]
    fn test_get_command_no_command(cmd_str: &str, invalid_portion: &str) {
        let temp_file = NamedTempFile::new().unwrap();
        temp_file
            .reopen()
            .unwrap()
            .write_all(TOML_COMMAND_DATA)
            .unwrap();
        let result = get_command(temp_file.path(), cmd_str.split_whitespace().collect());
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandParseError::CommandNotFoundError(s) => assert_eq!(s, invalid_portion),
            err => panic!(
                "Expected `CommandParseError::CommandNotFoundError`, got {:?}",
                err
            ),
        }
    }

    #[test]
    fn test_get_command_empty_command() {
        let temp_file = NamedTempFile::new().unwrap();
        temp_file
            .reopen()
            .unwrap()
            .write_all(TOML_COMMAND_DATA)
            .unwrap();
        let result = get_command(temp_file.path(), vec!["baz"]);
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandParseError::CommandContentInvalid(InvalidContentReason::MissingKey(key)) => {
                assert_eq!(key, "command")
            }
            err => panic!(
                "Expected `CommandParseError::CommandNotFoundError`, got {:?}",
                err
            ),
        }
    }

    #[test]
    fn test_get_command_not_table() {
        let temp_file = NamedTempFile::new().unwrap();
        temp_file
            .reopen()
            .unwrap()
            .write_all(TOML_COMMAND_DATA)
            .unwrap();
        let result = get_command(temp_file.path(), "foo qux".split_whitespace().collect());
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandParseError::CommandContentInvalid(InvalidContentReason::NotTomlTable(
                key,
                value,
            )) => {
                assert_eq!(key, "qux");
                if let Value::String(_) = value {
                } else {
                    panic!("Expected a `Value::Table` but got {}", value)
                }
            }
            err => panic!(
                "Expected wrapped `InvalidContentReason::NotTomlTable`, but got {:?}",
                err
            ),
        }
    }

    #[test]
    fn test_get_command_not_string() {
        let temp_file = NamedTempFile::new().unwrap();
        temp_file
            .reopen()
            .unwrap()
            .write_all(TOML_COMMAND_DATA)
            .unwrap();
        let result = get_command(temp_file.path(), "foo".split_whitespace().collect());
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandParseError::CommandContentInvalid(InvalidContentReason::NotTomlString(
                key,
                value,
            )) => {
                assert_eq!(key, "command");
                if let Value::String(_) = value {
                    panic!("Expected a `Value::String` but got {}", value)
                }
            }
            err => panic!(
                "Expected wrapped `InvalidContentReason::NotTomlString`, but got {:?}",
                err
            ),
        }
    }
}
