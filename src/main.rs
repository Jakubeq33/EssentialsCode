/// Made by Kubusieq | Jakubeq33
/// Thanks for using EssentialsCode! ❤️❤️

mod ui;
mod parser;
mod fixer;
mod scanner;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ess",
    author,
    version,
    about = "EssentialsCode - Smart error fixer for developers",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "find-bug", visible_alias = "scan")]
    FindBug {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        #[arg(short, long)]
        lang: Option<String>,
    },

    #[command(name = "bug", visible_alias = "fix")]
    Bug {
        #[arg(trailing_var_arg = true, num_args = 1..)]
        error: Vec<String>,
    },

    #[command(name = "list")]
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    ui::print_banner();

    match cli.command {
        Commands::FindBug { path, lang } => {
            scanner::scan_project(&path, lang.as_deref())?;
        }
        Commands::Bug { error } => {
            let error_text = error.join(" ");
            if error_text.trim().is_empty() {
                ui::print_error("Please provide an error message");
                ui::print_hint("Usage: ess bug \"<paste your error here>\"");
                return Ok(());
            }
            fixer::analyze_error(&error_text)?;
        }
        Commands::List => {
            ui::print_supported_patterns();
        }
    }

    Ok(())
}
