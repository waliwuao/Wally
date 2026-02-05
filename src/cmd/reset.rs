use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let output = Command::new("git")
        .args(&["log", "--pretty=format:%h - %s", "-n", "20"])
        .output()
        .context("Failed to get git log")?;

    let stdout = String::from_utf8(output.stdout)?;
    let commits: Vec<&str> = stdout.lines().collect();

    if commits.is_empty() {
        println!("No commits found to reset to.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit to reset to (HARD reset)")
        .default(0)
        .items(&commits)
        .interact()
        .context("Failed to read selection")?;

    let selected_commit = commits[selection];
    let commit_hash = selected_commit
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid commit format"))?;

    println!("Resetting to {}...", commit_hash);

    let status = Command::new("git")
        .args(&["reset", "--hard", commit_hash])
        .status()
        .context("Failed to execute git reset")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    println!("Successfully reset to {}", commit_hash);

    Ok(())
}