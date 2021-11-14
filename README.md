# frish
A simple command line shell written in the Rust programming language.

## Features
* Simple syntax of the command line
* Builtin commands for basic tasks such as printing the arguments
* Builtin commands for working with directories
* Builtin commands for working with files
* Builtin commands for working with processes
* Support for subshells and running commands in the background
* Support for redirection of standard input and output
* Support for pipelines

Syntax of the command line is as simple as possible: the first word is always the name of a command followed by zero or more arguments. The last three arguments may specify redirection and running of the command in the background and must be in the following order: `\<_IN_FILE \>OUT_FILE &`.

## Install

Install Rust, clone the repo and run `cargo run`.

## Usage

### Basic commands
* `help` ... Prints list of builtin commands
* `name` ... Prints the shell name
* `name word` ... Sets the shell name
* `loglevel` ... Print the current logging level
* `loglevel level`... Sets the current loggin level (Error, Info, Debug)
* `print args` ... Print the arguments
* `echo args` ... Print the arguments and final newline character

### Directory manipulation
* `dir.change` ... Change the current directory (if no argument given then change to the root directory)
* `dir.where` ... Print the current working directory
* `dir.make` ... Make one or more directories
* `dir.remove` ... Remove one or more directories
* `dir.list` ... List files in the given directory
* `dir.inspect` ... Verbose listing of files in the given directory

### File manipulation
* `link.hard original new` ... Create hard link
* `link.soft original new` ... Create soft/symbolic link
* `link.read links` ... Print targets of given links
* `unlink files` ... Remove (unlink) given files
* `rename source dest` ... Rename file
* `cpcat source dest` ... Copy source file to dest (if source or dest is '-' then use standard input or output, respectively)

### Process manipulation
* `pid` ... Print PID of the current shell
* `ppid` ... Print PPID of the current shell
* `lastpid` ... Print PID of the last command run in the background
* `status` ... Print status of the last command
* `exit` ... Exit from the current shell
* `exit status` ... Exit from the current shell with the given status
* `depth` ... Print depth of the current subshell
* `subshell` ... Run a subshell with the given command, e.g. `subshell echo 42`
* `pipes` ... Create a pipeline, e.g., `pipes "cat /etc/passwd" "cut -d: -f7" "uniq" "sort" "uniq -c"`

## Trivia
The original shell specification stems from an assignment in the Operating systems course at Faculity of Computer and Information Science, University of Ljubljana where students must write a command line shell in the C programming language. Hence, we have a simple syntax of the command line and a selected set of commands for file, directory and process manipulation including standard input/output redirection, running commands in the background, and pipeline manipulation.
