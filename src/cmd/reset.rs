use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};

pub fn run() -> Result<()> {
    let modes = vec![
        "Undo Recent Actions (Reflog) - Find 'lost' commits",
        "Reset to Specific Commit (Log) - Go back in history",
    ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select reset mode")
        .default(0)
        .items(&modes)
        .clear(true)
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
        .with_prompt("Select action to UNDO")
        .default(0)
        .items(&entries)
        .clear(true)
        .interact()?;

    ask_reset_type_and_execute(entries[selection])
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
        .with_prompt("Select commit to reset to")
        .default(0)
        .items(&entries)
        .clear(true)
        .interact()?;

    ask_reset_type_and_execute(entries[selection])
}

fn ask_reset_type_and_execute(entry: &str) -> Result<()> {
    let hash = entry
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid entry format"))?;

    let options = vec![
        "Soft: Keep my changes (Safe. Changes stay in 'Staging Area', ready to commit)",
        "Mixed: Keep my files, but unstage (Safe. Changes stay in files, but not 'Added')",
        "Hard: Discard all changes (DANGEROUS! Files will be exactly like the target commit)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("How should we reset to {}?", hash))
        .default(0)
        .items(&options)
        .interact()?;

    let (mode_arg, is_dangerous) = match selection {
        0 => ("--soft", false),
        1 => ("--mixed", false),
        2 => ("--hard", true),
        _ => unreachable!(),
    };

    if is_dangerous {
        let red = Style::new().red().bold();
        println!("{}", red.apply_to("WARNING: Hard reset will PERMANENTLY DELETE all uncommitted changes."));
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you absolutely sure you want to proceed?")
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    println!("Performing {} reset to {}...", mode_arg, hash);
    let status = execute_git(&["reset", mode_arg, hash])?;

    if status.success() {
        println!("{}", Style::new().green().apply_to(format!("Successfully reset to {}", hash)));
    } else {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    Ok(())
}