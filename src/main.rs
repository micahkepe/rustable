use clap::{CommandFactory, Parser};
use csv::Reader;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal},
};

use rustable::{Args, format_table};

/**
 * It's tabling time.
 */
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut reader: Reader<Box<dyn BufRead>> = match args.input {
        Some(path) => {
            let file = File::open(&path)?;
            Reader::from_reader(Box::new(BufReader::new(file)))
        }
        None => {
            // check if terminal
            if io::stdin().is_terminal() {
                // no piped input or file specified
                let mut cmd = Args::command();
                return Ok(cmd.print_help()?);
            }
            Reader::from_reader(Box::new(io::stdin().lock()))
        }
    };

    let align = args.align.unwrap_or_default();
    println!("{}", format_table(&mut reader, align)?);

    Ok(())
}
