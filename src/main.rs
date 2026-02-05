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
            cmd::new::run(project_name, template)?;
        }
        Commands::Context => {
            cmd::context::run()?;
        }
        Commands::List => {
            cmd::list::run()?;
        }
        Commands::Commit => {
            cmd::commit::run()?;
        }
        Commands::Branch => {
            cmd::branch::run()?;
        }
        Commands::Sync => {
            cmd::sync::run()?;
        }
        Commands::Reset => {
            cmd::reset::run()?;
        }
        Commands::Install { path } => {
            cmd::install::run(&path)?;
        }
        Commands::Uninstall { template_name } => {
            cmd::uninstall::run(&template_name)?;
        }
    }

    Ok(())
}