// Output formatting utilities for the CLI

use std::io::Write;

/// Output format for commands
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    /// Parse an output format string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Text,
        }
    }
}

/// Print a value as pretty JSON to stdout
pub fn print_json<T: serde::Serialize>(data: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// A simple column-aligned table printer
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    widths: Vec<usize>,
}

impl Table {
    /// Create a new table with the given headers
    pub fn new(headers: &[&str]) -> Self {
        let widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
            widths,
        }
    }

    /// Add a row of data (must have the same number of columns as headers)
    pub fn add_row(&mut self, cells: &[String]) {
        for (i, cell) in cells.iter().enumerate() {
            if i < self.widths.len() {
                self.widths[i] = self.widths[i].max(cell.len());
            }
        }
        self.rows.push(cells.to_vec());
    }

    /// Render the table to the given writer
    pub fn render(&self, writer: &mut dyn Write) -> std::io::Result<()> {
        // Print header
        for (i, header) in self.headers.iter().enumerate() {
            write!(writer, "{}", pad_string(header, self.widths[i]))?;
            if i < self.headers.len() - 1 {
                write!(writer, "  ")?;
            }
        }
        writeln!(writer)?;

        // Print separator
        for (i, width) in self.widths.iter().enumerate() {
            write!(writer, "{}", "-".repeat(*width))?;
            if i < self.headers.len() - 1 {
                write!(writer, "  ")?;
            }
        }
        writeln!(writer)?;

        // Print rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                write!(writer, "{}", pad_string(cell, self.widths[i]))?;
                if i < self.headers.len() - 1 {
                    write!(writer, "  ")?;
                }
            }
            writeln!(writer)?;
        }

        Ok(())
    }

    /// Render the table to stdout
    pub fn print(&self) {
        let mut stdout = std::io::stdout();
        self.render(&mut stdout).ok();
    }
}

fn pad_string(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}
