//! X Language Syntax Highlight Generator

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod generators;
mod model;
mod token_mapping;
mod utils;

/// Syntax highlight generator for X Language
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output directory for generated files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate all syntax definitions
    All,

    /// Generate VS Code syntax definition
    Vscode,

    /// Generate Vim syntax definition
    Vim,

    /// Generate Neovim Tree-sitter syntax definition
    Neovim,

    /// Generate Sublime Text syntax definition
    Sublime,

    /// Generate Emacs syntax definition
    Emacs,

    /// Generate JetBrains IDE syntax definition
    Jetbrains,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build syntax model from x-lexer tokens
    let syntax_model = token_mapping::build_syntax_model()?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&cli.output)?;

    match cli.command {
        Commands::All => {
            generators::vscode::generate(&syntax_model, &cli.output)?;
            generators::vim::generate(&syntax_model, &cli.output)?;
            generators::neovim::generate(&syntax_model, &cli.output)?;
            generators::sublime::generate(&syntax_model, &cli.output)?;
            generators::emacs::generate(&syntax_model, &cli.output)?;
            generators::jetbrains::generate(&syntax_model, &cli.output)?;
            println!("Generated all syntax definitions in {:?}", cli.output);
        }
        Commands::Vscode => {
            generators::vscode::generate(&syntax_model, &cli.output)?;
            println!("Generated VS Code syntax definition in {:?}", cli.output);
        }
        Commands::Vim => {
            generators::vim::generate(&syntax_model, &cli.output)?;
            println!("Generated Vim syntax definition in {:?}", cli.output);
        }
        Commands::Neovim => {
            generators::neovim::generate(&syntax_model, &cli.output)?;
            println!("Generated Neovim Tree-sitter syntax definition in {:?}", cli.output);
        }
        Commands::Sublime => {
            generators::sublime::generate(&syntax_model, &cli.output)?;
            println!("Generated Sublime Text syntax definition in {:?}", cli.output);
        }
        Commands::Emacs => {
            generators::emacs::generate(&syntax_model, &cli.output)?;
            println!("Generated Emacs syntax definition in {:?}", cli.output);
        }
        Commands::Jetbrains => {
            generators::jetbrains::generate(&syntax_model, &cli.output)?;
            println!("Generated JetBrains IDE syntax definition in {:?}", cli.output);
        }
    }

    Ok(())
}
