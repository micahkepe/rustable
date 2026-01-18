#![doc = include_str!("../README.md")]
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

    // First pass to collect info for pretty-printing
    let mut max_single_col_bytes: Vec<usize> = vec![];
    if let Ok(headers) = reader.headers() {
        for col in headers.iter() {
            max_single_col_bytes.push(col.to_owned().len())
        }
    }
    let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>()?;
    for record in &records {
        for (i, elem) in record.iter().enumerate() {
            max_single_col_bytes[i] = max_single_col_bytes[i].max(elem.to_owned().len())
        }
    }

    // Second pass to pretty-print
    let col_maxes_total: usize = max_single_col_bytes.iter().sum();
    let horiztonal_bar_width =
        col_maxes_total + 2 * max_single_col_bytes.len() + max_single_col_bytes.len() + 1;

    if let Ok(headers) = reader.headers() {
        print!("| ");
        for (i, header) in headers.iter().enumerate() {
            print!("{}", pad_center(header, max_single_col_bytes[i]));
            if i != max_single_col_bytes.len() - 1 {
                print!(" | ")
            }
        }
        println!(" |");
        draw_horizontal_bar(horiztonal_bar_width);
    }

    for record in &records {
        print!("| ");
        for (i, elem) in record.iter().enumerate() {
            print!("{}", pad_center(elem, max_single_col_bytes[i]));
            if i != max_single_col_bytes.len() - 1 {
                print!(" | ")
            }
        }
        println!(" |");
    }

    Ok(())
}

fn draw_horizontal_bar(width: usize) {
    println!("|{}|", "-".repeat(width - 2));
}

fn pad_center(elem: &str, width: usize) -> String {
    let len = elem.len();
    let total_pad = width - len;
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}
