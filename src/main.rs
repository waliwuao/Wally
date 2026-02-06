mod cli;
mod cmd;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::New { project_name, template }) => {
            cmd::new::run(project_name, template)?;
        }
        Some(Commands::Context) => {
            cmd::context::run()?;
        }
        Some(Commands::List) => {
            cmd::list::run()?;
        }
        Some(Commands::Commit) => {
            cmd::commit::run()?;
        }
        Some(Commands::Branch) => {
            cmd::branch::run()?;
        }
        Some(Commands::Update) => {
            cmd::update::run()?;
        }
        Some(Commands::Reset) => {
            cmd::reset::run()?;
        }
        Some(Commands::Stats) => {
            cmd::stats::run()?;
        }
        Some(Commands::Install { path }) => {
            cmd::install::run(&path)?;
        }
        Some(Commands::Uninstall { template_name }) => {
            cmd::uninstall::run(&template_name)?;
        }
        Some(Commands::Help) => {
            cmd::help::run()?;
        }
        None => {
            cmd::help::run()?;
        }
    }

    Ok(())
}