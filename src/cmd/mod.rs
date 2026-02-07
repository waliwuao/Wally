pub mod add;
pub mod branch;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod menu;
pub mod new;
pub mod push;
pub mod reset;
pub mod stats;
pub mod tag;
pub mod uninstall;
pub mod update;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");

use anyhow::{Context, Result};
use console::Style;
use std::process::{Command, ExitStatus, Output};

pub fn execute_git(args: &[&str]) -> Result<ExitStatus> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .status()
        .context("Failed to execute git command")
}

pub fn execute_git_output(args: &[&str]) -> Result<Output> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")
}

pub fn print_git_cmd(args: &[&str]) {
    let cmd_style = Style::new().blue().bold();
    let symbol = Style::new().cyan().bold();
    println!("{} {}", symbol.apply_to(">"), cmd_style.apply_to(format!("git {}", args.join(" "))));
}

pub fn get_remotes() -> Result<Vec<String>> {
    let output = execute_git_output(&["remote"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

pub fn add_remote_workflow() -> Result<String> {
    use dialoguer::{theme::ColorfulTheme, Input};
    
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Remote name (e.g., origin, github, gitee)")
        .default("origin".into())
        .interact_text()?;

    let url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("URL for remote '{}'", name))
        .interact_text()?;

    execute_git(&["remote", "add", &name, &url])?;
    Ok(name)
}