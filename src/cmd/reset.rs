use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let modes = vec![
        "Undo Recent Actions (Reflog)",
        "Reset to Specific Commit (Log)",
    ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select reset mode")
        .default(0)
        .items(&modes)
        .interact()
        .context("Failed to read mode selection")?;

    if mode_selection == 0 {
        handle_reflog_reset()
    } else {
        handle_log_reset()
    }
}

fn handle_reflog_reset() -> Result<()> {
    let output = Command::new("git")
        .args(&["reflog", "-n", "20", "--pretty=format:%h - %gs: %s"])
        .output()
        .context("Failed to get git reflog")?;

    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No recent actions found in reflog.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action to UNDO (Reset to state before/at this action)")
        .default(0)
        .items(&entries)
        .interact()
        .context("Failed to read selection")?;

    perform_reset(entries[selection])
}

fn handle_log_reset() -> Result<()> {
    let output = Command::new("git")
        .args(&["log", "--pretty=format:%h - %s (%cr)", "-n", "20"])
        .output()
        .context("Failed to get git log")?;

    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No commits found to reset to.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit to reset to (HARD reset)")
        .default(0)
        .items(&entries)
        .interact()
        .context("Failed to read selection")?;

    perform_reset(entries[selection])
}

fn perform_reset(entry: &str) -> Result<()> {
    let hash = entry
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid entry format"))?;

    println!("Performing HARD reset to {}...", hash);

    let status = Command::new("git")
        .args(&["reset", "--hard", hash])
        .status()
        .context("Failed to execute git reset")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    println!("Successfully reset to {}", hash);
    Ok(())
}