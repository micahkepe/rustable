# `rustable`

A minimal dependency CLI and library for tablifying semi-structured content into
pretty-printed Markdown tables.

## Usage

```text
Tablify semi-structured content into pretty-printed Markdown tables

Usage: rustable [OPTIONS] [INPUT]

Arguments:
  [INPUT]  The input file path contents to tablify. If omitted, attempts to read from `stdin`

Options:
  -a, --align <ALIGN>  Alignment of the elements in the table [possible values: center, left, right]
  -c, --color <COLOR>  Color option for the output [default: auto] [possible values: auto, always, never]
      --no-color       Alias for `--color never`
  -h, --help           Print help (see more with '--help')
  -V, --version        Print version
```
