#![doc = include_str!("../README.md")]
use clap::ValueEnum;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

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

/// Format a CSV table into a pretty-printed Markdown table.
///
/// TODO: make input generic over tabular data source
pub fn format_table<R: io::Read>(
    reader: &mut R,
    align: ColumnAlign,
) -> anyhow::Result<String> {
    let mut csv_reader = csv::Reader::from_reader(reader);

    // First pass to collect info for pretty-printing
    let mut max_single_col_display_width: Vec<usize> = vec![];
    if let Ok(headers) = csv_reader.headers() {
        for col in headers.iter() {
            max_single_col_display_width.push(col.width_cjk())
        }
    }
    let records: Vec<csv::StringRecord> =
        csv_reader.records().collect::<Result<_, _>>()?;
    for record in &records {
        for (i, elem) in record.iter().enumerate() {
            max_single_col_display_width[i] =
                max_single_col_display_width[i].max(elem.width_cjk())
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

    if let Ok(headers) = csv_reader.headers() {
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
        write_md_horizontal_bar(&mut out, horiztonal_bar_width)?;
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

/// Write a horizontal bar in Markdown with the given width with newline to the given buffer.
fn write_md_horizontal_bar<W: std::io::Write>(
    mut buffer: W,
    width: usize,
) -> std::io::Result<()> {
    writeln!(&mut buffer, "|{}|", "-".repeat(width - 2))?;
    Ok(())
}

/// Pad a string to the center with spaces (for center-aligned columns).
fn pad_center(elem: &str, width: usize) -> String {
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = total_pad / 2;
    let right_pad = total_pad.saturating_sub(left_pad);
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}

/// Pad a string to the left with spaces (for right-aligned columns).
fn pad_left(elem: &str, width: usize) -> String {
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = total_pad;
    let right_pad = 0;
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}

/// Pad a string to the right with spaces (for left-aligned columns).
fn pad_right(elem: &str, width: usize) -> String {
    let len = elem.width_cjk();
    let total_pad = width.saturating_sub(len);
    let left_pad = 0;
    let right_pad = total_pad;
    format!("{}{}{}", " ".repeat(left_pad), elem, " ".repeat(right_pad))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn normal_csv_table() {
        let input = "\
Username,Identifier,First name,Last name
booker12,9012,Rachel,Booker
grey07,2070,Laura,Grey";
        let mut reader = Cursor::new(input);
        let actual = format_table(&mut reader, ColumnAlign::Center).unwrap();
        let expected = "\
| Username | Identifier | First name | Last name |
|------------------------------------------------|
| booker12 |    9012    |   Rachel   |  Booker   |
|  grey07  |    2070    |   Laura    |   Grey    |
";
        assert_eq!(
            actual, expected,
            "\nexpected=\n{}\n\ngot=\n{}",
            expected, actual
        );
    }
}
