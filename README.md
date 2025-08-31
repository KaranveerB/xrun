# xrun
A program to help manage scripts/alias with a flexible subcommand like
structure.

The primary motivation for this is to better manage my growing alias file where
I have multiple relates aliases with the same prefix (which is better grouped
under a subcommand) and a more convenient place to store cool one liners.

This program is in development and many features/changes may be made over time.

See my [dotfiles](https://github.com/KaranveerB/Dotfiles/blob/master/.config/xrun/command.toml)
for a real-world example of its usage.

## Configuring
The program reads from `$XDG_CONFIG_HOME/xrun/command.toml` for commands. This
is usually `~/.config/xrun/command.toml`.

Each command can have the following keys
* `command`: (optional for subcommands) A string of the command to execute.
* `desc`: (optional) description of the command/subcommand.

Any other key is treated as the name of the command. Commands can be nested to
create a subcommand in command tree structure.

For example
```toml
[msg]
bid-farewell = { command = "echo bye", desc = "says bye" }

[msg.greet]
desc = "greets the user"
command = "xrun msg greet kind"
casual = { command = "echo sup", desc = "says sup" }
kind = { command = "echo hi", desc = "says hi" }
```

You can then use the program as follows
```sh
> xrun msg greet
hi
> xrun msg greet kind
hi
> xrun msg greet casual
sup
> xrun msg bid-farewell
bye
> xrun msg --help
usage: xrun msg [command]
commands:
    bid-farewell: says bye
    greet: greets the user
> xrun msg greet --help
usage: xrun msg greet [command]
greets the user

commands:
    casual: says sup
    kind: says hi
```

## Passthrough
Using the `--passthrough` flag prints the shell commands to stdout.
This can be used to run the command directly in the current shell and avoid any
weirdness/performance hits of spawning a new shell as a child of `xrun`.

An exit code of `125` is returned if a shell command was returned.
This can be used in bash like shells (or whatever equivalent for your shell) as
follows:

```bash
function xrun() {
  output=$("$@" 2>&1)
  if [ $? -eq 125 ]; then
    eval "$output"
  else
    echo "$output"
  fi
}
```

