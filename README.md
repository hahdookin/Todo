# Todo
Track and organize tasks in the command-line.

## Installation
Clone into a folder and run `cargo build`.

Add path to `todo` to your env if you'd like.

## Usage
`todo <command> [<args>]`

Try `todo --help` for more.

## Examples
Adding an entry:

`todo add "Homework" "10/23/2021 11:59 pm" "Programming project!"`

Listing all entries:

`todo list` (or just `todo`)
```
Homework:
[0] Due: Sat Oct 23 11:59 PM: Programming project!
```

Modifying an entry:

`todo mod 0 group="School Projects"`

Removing an entry (task complete!):

`todo del 0`

## Inspiration
There were a few factors influencing me to take on this project:
1. I wanted to start a simple project to learn Rust.
2. I spend all my time in the command-line and wanted a simple tool to keep track of my tasks.

## What's to come
- [ ] Configuration settings (akin to .vimrc)
- [ ] Cross platform support (Mac, Windows)

