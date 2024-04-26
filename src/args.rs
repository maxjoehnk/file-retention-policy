use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Config File
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<SubCommand>,
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SubCommand {
    Simulate {
        /// Path to simulate retention policy for
        path: PathBuf,
        /// Textfile with one filename per line
        #[arg(long)]
        input: Option<PathBuf>
    }
}
