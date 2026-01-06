/// Made by Kubusieq | Jakubeq33
/// Thanks for using EssentialsCode!
mod config;
mod fixer;
mod parser;
mod scanner;
mod ui;

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
    /// Scan a project for errors
    #[command(name = "find-bug", visible_alias = "scan")]
    FindBug {
        /// Path to the project directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Specific language to check
        #[arg(short, long)]
        lang: Option<String>,
    },

    /// Analyze a specific error message
    #[command(name = "bug", visible_alias = "fix")]
    Bug {
        /// The error message to analyze
        #[arg(trailing_var_arg = true, num_args = 1..)]
        error: Vec<String>,
    },

    /// List supported error patterns
    #[command(name = "list")]
    List,

    /// Initialize a configuration file
    #[command(name = "init")]
    Init {
        /// Create global config instead of local
        #[arg(long)]
        global: bool,
    },
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
        Commands::Init { global } => {
            init_config(global)?;
        }
    }

    Ok(())
}

fn init_config(global: bool) -> Result<()> {
    use config::Config;

    let config_path = if global {
        Config::global_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
    } else {
        Config::project_config_path(&std::env::current_dir()?)
    };

    if config_path.exists() {
        ui::print_warning(&format!(
            "Config file already exists: {}",
            config_path.display()
        ));
        ui::print_hint("Delete it first if you want to create a new one");
        return Ok(());
    }

    // Create parent directories if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write example config
    std::fs::write(&config_path, Config::example_config())?;

    ui::print_info(&format!("Created config file: {}", config_path.display()));
    ui::print_hint("Edit this file to customize EssentialsCode behavior");

    Ok(())
}
