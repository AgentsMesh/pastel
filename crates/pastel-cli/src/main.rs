mod commands;
mod pipeline;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pastel", version, about = "Pastel - Design as Code")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile and render a .pastel file
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Validate a .pastel file without rendering
    Check {
        file: PathBuf,
    },
    /// Show the node tree (dry-run)
    Plan {
        file: PathBuf,
    },
    /// Format a .pastel source file
    Fmt {
        file: PathBuf,
    },
    /// Show IR summary
    Inspect {
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Start live preview server
    Serve {
        file: PathBuf,
        #[arg(long, default_value = "3210")]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build { file, output } => commands::build::run(&file, &output),
        Command::Check { file } => commands::check::run(&file),
        Command::Plan { file } => commands::plan::run(&file),
        Command::Fmt { file } => commands::fmt::run(&file),
        Command::Inspect { file, json } => commands::inspect::run(&file, json),
        Command::Serve { file, port } => commands::serve::run(&file, port),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
