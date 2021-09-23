# Todo
Track and organize tasks in the command-line.

### Installation
Clone into a folder and run `cargo build`.

Add path to `todo` to your env if you'd like.

### Usage
`todo <command> [<args>]`

Try `todo --help` for more.

### Examples
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

