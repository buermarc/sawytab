# swaytab - Change the focus to a specific window in sway

## Description

Use fzf or any other filter tool to chose a specific window which should be
focused. The filter command can be set via configuration or via command line
arguments.

Different tools like [swayr](https://git.sr.ht/~tsdh/swayr) already implement
this behaviour, but I discovered it too late. If you search for a tool,
[swayr](https://git.sr.ht/~tsdh/swayr) provides way more features, but it
requires a running daemon.


## Installation

Clone the repo and build the release target

```shell
$ git clone https://github.com/buermarc/swaytab swaytab
$ cd swaytab
$ cargo build --release
$ ./target/release/swaytab # <- Executable
```
Another way would be using `cargo install`. 

```shell
$ git clone https://github.com/buermarc/swaytab swaytab
$ cd swaytab
$ cargo install --path .
$ which swaytab
~/.cargo/bin/swaytab
```

Using `cargo install` requires that the `$/HOME/.cargo/bin` directory is in the
PATH variable.

## TODO

- Allow changing to workspaces
- Different parsing of command line arguments
