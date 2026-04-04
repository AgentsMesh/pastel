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
        /// Render only the specified page (by name)
        #[arg(long)]
        page: Option<String>,
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
    /// Generate code from a .pastel file
    Gen {
        file: PathBuf,
        /// Output format: tokens, html, react
        #[arg(long)]
        format: String,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Lint a .pastel file: check design values against token definitions
    Lint {
        /// .pastel file to check
        file: PathBuf,
        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build { file, output, page } => commands::build::run(&file, &output, page.as_deref()),
        Command::Check { file } => commands::check::run(&file),
        Command::Plan { file } => commands::plan::run(&file),
        Command::Fmt { file } => commands::fmt::run(&file),
        Command::Inspect { file, json } => commands::inspect::run(&file, json),
        Command::Serve { file, port } => commands::serve::run(&file, port),
        Command::Gen { file, format, output } => commands::gen::run(&file, &format, &output),
        Command::Lint { file, format } => commands::lint::run(&file, &std::path::PathBuf::new(), &format),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
