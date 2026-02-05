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
    Context,
    List,
    Add {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        files: Vec<String>,
    },
    Commit,
    Push {
        url: Option<String>,
    },
    Branch,
    Reset,
    Install {
        path: String,
    },
    Uninstall {
        template_name: String,
    },
}