use crate::cmd::{execute_git, execute_git_output, print_step};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};

pub fn run() -> Result<()> {
    print_step("Reset Options");
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
    let output = execute_git_output(&["reflog", "-n", "20", "--pretty=format:%h - %gs: %s"])?;
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
        .interact()?;

    perform_reset(entries[selection])
}

fn handle_log_reset() -> Result<()> {
    let output = execute_git_output(&["log", "--pretty=format:%h - %s (%cr)", "-n", "20"])?;
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
        .interact()?;

    perform_reset(entries[selection])
}

fn perform_reset(entry: &str) -> Result<()> {
    let hash = entry
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid entry format"))?;

    println!("Performing HARD reset to {}...", hash);

    let status = execute_git(&["reset", "--hard", hash])?;

    if !status.success() {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    println!("Successfully reset to {}", hash);
    Ok(())
}