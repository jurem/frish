* uname to set mode for new files (e.g. stdout redirection)
* tokenize: "escape squences", 'no escape sequences'
* support spaces in filename for redirection, e.g. > "a.txt" - two tokens
* change dir list to use only nix functions
* commands: rand.int, rand.int range, rand.XXX, rand.seed seed
* Refactor Error, Result: make own Error type and handle different errors separately.
* do_cpcat: use only nix, no high-level fun
* aliases
* variables
* Check the logger: RUST_LOG=Debug seems to work with loglevel command. If RUST_LOG=Error then loglevel setting is ignored.
