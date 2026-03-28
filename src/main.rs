mod format;

use std::io::{self, BufRead, Write};

use clap::Parser;
use csv::ReaderBuilder;
use format::{parse_template, render};

/// Apply a format string to every row of CSV/TSV (or other delimited) input.
///
/// # Field references
///
///   {N}          — value of the Nth field (1-based)
///   {name}       — value of the field whose header is "name" (requires -H)
///   {N:default}  — field value, or "default" when the field is empty
///   {?N:text}    — include "text" only when field N is non-empty
///                  ("text" may itself contain {…} references)
///   {{  }}       — literal `{` / `}`
///
/// # Examples
///
///   echo "Alice,30" | csvfmt "Hello {1}, you are {2} years old."
///
///   pbpaste | csvfmt -H "Dear {first_name} {last_name},"
///
///   cat data.tsv | csvfmt -t "{1}\t{2}{?3: (note: {3})}"
#[derive(Parser, Debug)]
#[command(author, version, about, verbatim_doc_comment)]
struct Args {
    /// Format template to apply to each row.
    ///
    /// Use {N} for 1-based field index, {name} for named fields (requires -H),
    /// {N:default} for a fallback value, and {?N:text} for conditional text.
    #[arg(required = true)]
    template: String,

    /// Input file path (default: read from stdin).
    #[arg(short, long, value_name = "FILE")]
    input: Option<String>,

    /// Field delimiter character (default: `,`).
    ///
    /// Use `\t` or `tab` to specify a literal tab.  See also `--tsv`.
    #[arg(short, long, value_name = "CHAR")]
    delimiter: Option<String>,

    /// Use tab as the field delimiter (shorthand for `-d '\t'`).
    #[arg(short, long, conflicts_with = "delimiter")]
    tsv: bool,

    /// Treat the first row as a header row (enables {name} references).
    #[arg(short = 'H', long)]
    header: bool,

    /// Trim leading/trailing whitespace from each field.
    #[arg(long)]
    trim: bool,

    /// Skip rows where every field is empty.
    #[arg(short, long)]
    skip_empty: bool,
}

fn main() {
    let args = Args::parse();

    // Resolve the delimiter byte.
    let delim: u8 = if args.tsv {
        b'\t'
    } else if let Some(ref d) = args.delimiter {
        parse_delimiter(d).unwrap_or_else(|e| {
            eprintln!("csvfmt: {e}");
            std::process::exit(1);
        })
    } else {
        b','
    };

    // Parse the format template.
    let segments = parse_template(&args.template).unwrap_or_else(|e| {
        eprintln!("csvfmt: invalid template: {e}");
        std::process::exit(1);
    });

    // Open input.
    let input: Box<dyn BufRead> = if let Some(path) = &args.input {
        let file = std::fs::File::open(path).unwrap_or_else(|e| {
            eprintln!("csvfmt: cannot open '{}': {e}", path);
            std::process::exit(1);
        });
        Box::new(io::BufReader::new(file))
    } else {
        Box::new(io::BufReader::new(io::stdin()))
    };

    let mut reader = ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false) // We handle the header row ourselves.
        .flexible(true)
        .from_reader(input);

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut headers: Option<Vec<String>> = None;
    let mut first = true;

    for result in reader.records() {
        let record = result.unwrap_or_else(|e| {
            eprintln!("csvfmt: read error: {e}");
            std::process::exit(1);
        });

        let mut fields: Vec<String> = record
            .iter()
            .map(|f| if args.trim { f.trim().to_string() } else { f.to_string() })
            .collect();

        // First row → store as headers when -H was given.
        if first && args.header {
            headers = Some(fields);
            first = false;
            continue;
        }
        first = false;

        // Skip entirely-empty rows when requested.
        if args.skip_empty && fields.iter().all(|f| f.is_empty()) {
            continue;
        }

        // Pad the record so that out-of-bounds references yield "".
        // (The csv crate already does this with `flexible`, but be explicit.)
        if fields.is_empty() {
            fields.push(String::new());
        }

        let line = render(&segments, &fields, headers.as_deref());
        writeln!(out, "{line}").unwrap_or_else(|e| {
            // Broken pipe is normal (e.g. `… | head`); exit quietly.
            if e.kind() == io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
            eprintln!("csvfmt: write error: {e}");
            std::process::exit(1);
        });
    }
}

/// Convert a user-supplied delimiter string to a single byte.
fn parse_delimiter(s: &str) -> Result<u8, String> {
    let s = match s {
        "\\t" | "tab" => "\t",
        other => other,
    };
    let chars: Vec<char> = s.chars().collect();
    match chars.as_slice() {
        [c] if c.len_utf8() == 1 => Ok(*c as u8),
        _ => Err(format!("delimiter must be a single ASCII character, got {s:?}")),
    }
}
