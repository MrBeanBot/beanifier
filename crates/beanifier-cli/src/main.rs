//! `beanify` — Mr-Beanify any path recursively into Mr-Bean-speak.

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use beanifier_cli::{run_stdout, Cli};

fn main() -> ExitCode {
    match real_main() {
        Ok(had_errors) => {
            if had_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("beanify: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<bool> {
    let cli = Cli::parse();
    let summary = run_stdout(&cli)?;
    eprintln!(
        "beanify: {} processed, {} skipped, {} errored",
        summary.processed, summary.skipped, summary.errors
    );
    Ok(summary.errors > 0)
}
