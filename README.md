# `MVC` (Mivio version control) is a utility written in Rust that provides a snapshot system that allows you to roll back code.
## Features:
1. Very lightweight: unlike similar projects, it's written in only ~350 lines of code (the binary file weighs ~750 KB).
2. Easy to use: to work with MVC, you only need four commands (mvc [init, save <message>, return <id>, log]).
## Usage
init repository:
```bash
$ mvc init
Repository initialized! Please execute "mvc save Initial" for create first snapshot!
[WARNING] set information about you
    mvc cfg name your_name
    mvc cfg email your@email
```
configure information:
```bash
$ mvc cfg name mivio
Success!
$ mvc cfg email miviodev@gmail.com
Success!
```
save:
```bash
$ mvc save Initial
Saved!
```
log:
```bash
$ mvc log
Snapshot ID: 1
Hash:        "abc123"
Message:     "Initial"
Email:       "miviodev@gmail.com"
Username:    "mivio"
---
```
# Building from source

You’ll also need `cargo` to build from source:

```bash
cargo build -r
```
The binary will be located in `./target/release/`.

---

licensed under the MIT License 
