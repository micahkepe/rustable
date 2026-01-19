use clap::{CommandFactory, Parser};
use std::{
    fs::File,
    io::{self, BufReader, IsTerminal},
    path::PathBuf,
};

use rustable::{ColumnAlign, format_table};

/// Tablify semi-structured content into pretty-printed Markdown tables.
#[derive(Parser, Debug)]
#[command(name = "rustable", version, about, long_about = None)]
struct Args {
    /// The input file path contents to tablify. If omitted, attempts to read from `stdin`.
    input: Option<PathBuf>,

    /// Alignment of the elements in the table.
    #[arg(short, long)]
    align: Option<ColumnAlign>,
}

/**
 * It's tabling time.
 */
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let align = args.align.unwrap_or_default();

    let output = match args.input {
        Some(path) => {
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            format_table(&mut reader, align)?
        }
        None => {
            // check if terminal
            if io::stdin().is_terminal() {
                // no piped input or file specified
                let mut cmd = Args::command();
                return Ok(cmd.print_help()?);
            }
            format_table(&mut io::stdin().lock(), align)?
        }
    };

    print!("{}", output);

    Ok(())
}
