#![doc = include_str!("../README.md")]
use clap::ValueEnum;
use std::io;
use termcolor::{Color, ColorSpec};
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

/// Available options for controlling whether to use color in the output.
#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum ColorOpt {
    /// Use color if the output is a terminal.
    #[default]
    Auto,
    /// Always use color.
    Always,
    /// Never use color.
    Never,
}

impl From<ColorOpt> for termcolor::ColorChoice {
    fn from(value: ColorOpt) -> Self {
        match value {
            ColorOpt::Auto => termcolor::ColorChoice::Auto,
            ColorOpt::Always => termcolor::ColorChoice::Always,
            ColorOpt::Never => termcolor::ColorChoice::Never,
        }
    }
}

/// Format a CSV table into a pretty-printed Markdown table.
///
/// TODO: make input generic over tabular data source
pub fn format_table<R, W>(
    reader: &mut R,
    out: &mut W,
    align: ColumnAlign,
) -> anyhow::Result<()>
where
    R: io::Read,
    W: termcolor::WriteColor,
{
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
        write!(out, "| ")?;
        for (i, header) in headers.iter().enumerate() {
            let mut header_spec = ColorSpec::new();
            header_spec.set_fg(Some(Color::Cyan)).set_bold(true);
            write_colored(
                out,
                &header_spec,
                &align_fn(header, max_single_col_display_width[i]),
            )?;
            if i != max_single_col_display_width.len() - 1 {
                write!(out, " | ")?
            }
        }
        writeln!(out, " |")?;
        write_md_horizontal_bar(out, horiztonal_bar_width)?;
    }

    for record in &records {
        write!(out, "| ")?;
        for (i, elem) in record.iter().enumerate() {
            write!(
                out,
                "{}",
                align_fn(elem, max_single_col_display_width[i])
            )?;
            if i != max_single_col_display_width.len() - 1 {
                write!(out, " | ")?
            }
        }
        writeln!(out, " |")?;
    }

    Ok(())
}

/// Helper to write colored text to a `WriteColor` instance.
///
/// Args:
/// * `out`: the `WriteColor` instance to write to.
/// * `spec`: the `ColorSpec` to use for the text.
/// * `text`: the text to write.
///
/// This replaces manually writing `out.set_color(spec)?; write!(out, "{text}")?; out.reset()?;`
fn write_colored<W: termcolor::WriteColor>(
    out: &mut W,
    spec: &termcolor::ColorSpec,
    text: &str,
) -> std::io::Result<()> {
    out.set_color(spec)?;
    write!(out, "{text}")?;
    out.reset()?;
    Ok(())
}

/// Write a horizontal bar in Markdown with the given width with newline to the given buffer.
fn write_md_horizontal_bar<W: termcolor::WriteColor>(
    out: &mut W,
    width: usize,
) -> std::io::Result<()> {
    writeln!(out, "|{}|", "-".repeat(width - 2))?;
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
    use termcolor::NoColor;

    use super::*;

    #[test]
    fn normal_csv_table() {
        let input = "\
Username,Identifier,First name,Last name
booker12,9012,Rachel,Booker
grey07,2070,Laura,Grey";
        let mut reader = Cursor::new(input);
        let mut out = NoColor::new(Vec::new());
        format_table(&mut reader, &mut out, ColumnAlign::Center).unwrap();
        let actual = String::from_utf8(out.into_inner()).unwrap();
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
