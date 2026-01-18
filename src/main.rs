#![doc = include_str!("../README.md")]
use clap::{CommandFactory, Parser, ValueEnum};
use csv::Reader;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal},
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

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

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
enum ColumnAlign {
    #[default]
    Center,
    Left,
    Right,
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
    let mut max_single_col_display_width: Vec<usize> = vec![];
    if let Ok(headers) = reader.headers() {
        for col in headers.iter() {
            max_single_col_display_width.push(col.width_cjk())
        }
    }
    let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>()?;
    for record in &records {
        for (i, elem) in record.iter().enumerate() {
            max_single_col_display_width[i] = max_single_col_display_width[i].max(elem.width_cjk())
        }
    }

    // Second pass to pretty-print
    let col_maxes_total: usize = max_single_col_display_width.iter().sum();
    let horiztonal_bar_width = col_maxes_total
        + 2 * max_single_col_display_width.len() // space on left/right
        + max_single_col_display_width.len() // '|' separator for each
        + 1;

    let align_fn = match args.align {
        Some(align) => match align {
            ColumnAlign::Center => pad_center,
            ColumnAlign::Left => pad_right,
            ColumnAlign::Right => pad_left,
        },
        None => pad_center,
    };

    if let Ok(headers) = reader.headers() {
        print!("| ");
        for (i, header) in headers.iter().enumerate() {
            print!("{}", align_fn(header, max_single_col_display_width[i]));
            if i != max_single_col_display_width.len() - 1 {
                print!(" | ")
            }
        }
        println!(" |");
        draw_horizontal_bar(horiztonal_bar_width);
    }

    for record in &records {
        print!("| ");
        for (i, elem) in record.iter().enumerate() {
            print!("{}", align_fn(elem, max_single_col_display_width[i]));
            if i != max_single_col_display_width.len() - 1 {
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
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = total_pad / 2;
    let right_pad = total_pad.saturating_sub(left_pad);
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}

fn pad_left(elem: &str, width: usize) -> String {
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = total_pad;
    let right_pad = 0;
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}

fn pad_right(elem: &str, width: usize) -> String {
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = 0;
    let right_pad = total_pad;
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}
