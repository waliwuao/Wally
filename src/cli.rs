use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wally")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    New {
        project_name: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
    },
    Context,
    List,
    Add,
    Commit,
    Push,
    Branch,
    Update,
    Reset,
    Stats,
    Tag,
    Install {
        path: String,
    },
    Uninstall {
        template_name: String,
    },
}