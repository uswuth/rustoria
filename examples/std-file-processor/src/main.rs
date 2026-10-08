use std::path::PathBuf;
use std::process::ExitCode;

use std_file_processor::{count_path, format_report, ProcessingError};

const USAGE: &str = "\
Usage: std-file-processor <PATH> [--top N] [--save FILE]

Counts word frequencies in a text file, or in every .txt file of a
directory, and prints a small report.

Arguments:
  <PATH>       A .txt file or a directory containing .txt files

Options:
  --top N      Show the N most frequent words [default: 10]
  --save FILE  Also write the report to FILE
  -h, --help   Print this help";

struct Args {
    input: PathBuf,
    top: usize,
    save: Option<PathBuf>,
}

/// Outcome of command-line parsing: run with args, print help and succeed,
/// or report a usage error and fail with exit code 2.
enum Cli {
    Run(Args),
    Help(String),
    Usage(String),
}

fn parse_args(argv: &[String]) -> Cli {
    let mut input = None;
    let mut top = 10;
    let mut save = None;

    let mut iter = argv.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--top" => {
                let Some(value) = iter.next() else {
                    return Cli::Usage("--top requires a value".to_string());
                };
                match value.parse::<usize>() {
                    Ok(n) if n >= 1 => top = n,
                    _ => {
                        return Cli::Usage(format!(
                            "--top must be a positive integer, got {value:?}"
                        ));
                    }
                }
            }
            "--save" => {
                let Some(value) = iter.next() else {
                    return Cli::Usage("--save requires a value".to_string());
                };
                save = Some(PathBuf::from(value));
            }
            "-h" | "--help" => return Cli::Help(USAGE.to_string()),
            other if other.starts_with('-') => {
                return Cli::Usage(format!("unknown flag {other:?}\n\n{USAGE}"));
            }
            other => {
                if input.is_some() {
                    return Cli::Usage(format!("unexpected extra argument {other:?}\n\n{USAGE}"));
                }
                input = Some(PathBuf::from(other));
            }
        }
    }

    match input {
        Some(input) => Cli::Run(Args { input, top, save }),
        None => Cli::Usage(format!("missing <PATH> argument\n\n{USAGE}")),
    }
}

fn run(args: Args) -> Result<(), ProcessingError> {
    let counts = count_path(&args.input)?;
    let report = format_report(&counts, args.top);
    print!("{report}");

    if let Some(path) = args.save {
        std::fs::write(&path, &report)?;
        println!("Report saved to {}", path.display());
    }

    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Cli::Run(args) => args,
        Cli::Help(text) => {
            // --help is a successful request for information: stdout, exit 0.
            println!("{text}");
            return ExitCode::SUCCESS;
        }
        Cli::Usage(message) => {
            // Usage errors are failures: stderr, exit 2 (conventional CLI code).
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
