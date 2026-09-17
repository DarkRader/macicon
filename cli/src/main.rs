//! Command-line interface entrypoint for macicon.

use clap::Parser;
use macicon::cli::args::Cli;
use macicon::cli::commands::run;

fn main() {
    let args = Cli::parse();
    if let Err(err) = run(args) {
        eprintln!("⚠️  Error: {}", err);
        std::process::exit(1);
    }
}
