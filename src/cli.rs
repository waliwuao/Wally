use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wally")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    New {
        project_name: String,
        #[arg(short, long)]
        template: Option<String>,
    },
    Tree,
    List,
}