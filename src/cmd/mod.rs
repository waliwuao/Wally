pub mod add;
pub mod branch;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod menu;
pub mod new;
pub mod reset;
pub mod stats;
pub mod tag;
pub mod uninstall;
pub mod update;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");

use anyhow::{Context, Result};
use console::Style;
use std::process::{Command, ExitStatus, Output};

/// Helper to execute git commands with unified printing style
pub fn execute_git(args: &[&str]) -> Result<ExitStatus> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .status()
        .context("Failed to execute git command")
}

/// Helper to execute git commands and capture output with unified printing style
pub fn execute_git_output(args: &[&str]) -> Result<Output> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")
}

/// Helper just to print the command (useful for mixed logic like piping)
pub fn print_git_cmd(args: &[&str]) {
    let cmd_style = Style::new().blue().bold();
    let symbol = Style::new().cyan().bold();
    println!("{} {}", symbol.apply_to(">"), cmd_style.apply_to(format!("git {}", args.join(" "))));
}

/// Helper for section headers
pub fn print_step(msg: &str) {
    let style = Style::new().magenta().bold();
    println!("\n{}", style.apply_to(format!("==> {}", msg)));
}