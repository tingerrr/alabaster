use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Manage and prepare unreleased and local typst packges and templates
#[derive(Debug, Parser)]
pub struct Args {
    /// The project root
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Make a package ready for submission to Typst universe
    #[command(aliases = ["p", "pkg"])]
    Package {
        /// Overwrite the output directory if it exists
        #[arg(long, short)]
        force: bool,

        /// The name to the output directory to generate
        output: PathBuf,
    },
}
