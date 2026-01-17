use clap::{CommandFactory, Parser};
use csv::Reader;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal},
    path::PathBuf,
};

/// Tablify semi-structured content into pretty-printed Markdown tables.
#[derive(Parser, Debug)]
#[command(name = "rustable", version, about, long_about = None)]
struct Args {
    /// The input file path contents to tablify. If omitted, attempts to read from `stdin`.
    input: Option<PathBuf>,
}

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

    if let Ok(h) = reader.headers() {
        println!("{:?}", h)
    }

    for r in reader.records() {
        let r = r?;
        println!("{:?}", r)
    }

    Ok(())
}
