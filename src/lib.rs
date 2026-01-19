#![doc = include_str!("../README.md")]
use clap::{Parser, ValueEnum};
use csv::Reader;
use std::{
    io::{BufRead, Write},
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

/// Tablify semi-structured content into pretty-printed Markdown tables.
#[derive(Parser, Debug)]
#[command(name = "rustable", version, about, long_about = None)]
pub struct Args {
    /// The input file path contents to tablify. If omitted, attempts to read from `stdin`.
    pub input: Option<PathBuf>,
    /// Alignment of the elements in the table.
    #[arg(short, long)]
    pub align: Option<ColumnAlign>,
}

/// The formatting alignment to use for column elements.
#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum ColumnAlign {
    /// Center-align all column elements.
    #[default]
    Center,
    /// Left-align all column elements.
    Left,
    /// Right-align all column elements.
    Right,
}

pub fn format_table(
    reader: &mut Reader<Box<dyn BufRead>>,
    align: ColumnAlign,
) -> anyhow::Result<String> {
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
    let mut out = Vec::new();
    let col_maxes_total: usize = max_single_col_display_width.iter().sum();
    let horiztonal_bar_width = col_maxes_total
        + 2 * max_single_col_display_width.len() // space on left/right
        + max_single_col_display_width.len() // '|' separator for each
        + 1;

    let align_fn = match align {
        ColumnAlign::Center => pad_center,
        ColumnAlign::Left => pad_right,
        ColumnAlign::Right => pad_left,
    };

    if let Ok(headers) = reader.headers() {
        write!(&mut out, "| ")?;
        for (i, header) in headers.iter().enumerate() {
            write!(
                &mut out,
                "{}",
                align_fn(header, max_single_col_display_width[i])
            )?;
            if i != max_single_col_display_width.len() - 1 {
                write!(&mut out, " | ")?
            }
        }
        writeln!(&mut out, " |")?;
        draw_horizontal_bar(horiztonal_bar_width);
    }

    for record in &records {
        write!(&mut out, "| ")?;
        for (i, elem) in record.iter().enumerate() {
            write!(
                &mut out,
                "{}",
                align_fn(elem, max_single_col_display_width[i])
            )?;
            if i != max_single_col_display_width.len() - 1 {
                write!(&mut out, " | ")?
            }
        }
        writeln!(&mut out, " |")?;
    }

    let s = String::from_utf8(out)?;
    Ok(s)
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
