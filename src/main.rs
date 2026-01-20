use clap::{ArgAction, CommandFactory, Parser};
use std::{
    fs::File,
    io::{self, BufReader, IsTerminal},
    path::PathBuf,
};
use termcolor::{ColorChoice, StandardStream};

use rustable::{ColorOpt, ColumnAlign, format_table};

/// Tablify semi-structured content into pretty-printed Markdown tables.
#[derive(Parser, Debug)]
#[command(name = "rustable", version, about, long_about = None)]
struct Args {
    /// The input file path contents to tablify. If omitted, attempts to read from `stdin`.
    input: Option<PathBuf>,

    /// Alignment of the elements in the table.
    #[arg(short, long)]
    align: Option<ColumnAlign>,

    /// Color option for the output.
    #[arg(short, long, default_value = "auto")]
    color: ColorOpt,

    /// Alias for `--color never`
    #[arg(long, action = ArgAction::SetTrue)]
    no_color: bool,
}

/**
 * It's tabling time.
 */
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let align = args.align.unwrap_or_default();
    let mut color_choice: ColorChoice = args.color.into();
    if args.no_color {
        color_choice = ColorChoice::Never;
    }

    // Terminal auto-detection
    // See: <https://docs.rs/termcolor/1.4.1/termcolor/index.html#detecting-presence-of-a-terminal>
    if color_choice == ColorChoice::Auto && !io::stdout().is_terminal() {
        color_choice = ColorChoice::Never;
    }
    let mut stdout = StandardStream::stdout(color_choice);

    match args.input {
        Some(path) => {
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            format_table(&mut reader, &mut stdout, align)?
        }
        None => {
            // check if terminal
            if io::stdin().is_terminal() {
                // no piped input or file specified
                let mut cmd = Args::command();
                return Ok(cmd.print_help()?);
            }
            format_table(&mut io::stdin().lock(), &mut stdout, align)?
        }
    };

    Ok(())
}
