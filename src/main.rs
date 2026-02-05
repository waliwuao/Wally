mod cli;
mod cmd;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::New { project_name, template } => {
            cmd::new::run(&project_name, template)?;
        }
        Commands::Tree => {
            cmd::tree::run()?;
        }
        Commands::List => {
            cmd::list::run()?;
        }
    }

    Ok(())
}