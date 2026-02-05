use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let actions = vec!["Switch Branch", "Create New Branch", "Delete Branch"];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch action")
        .default(0)
        .items(&actions)
        .interact()
        .context("Failed to read selection")?;

    match selection {
        0 => switch_branch(),
        1 => create_branch(),
        2 => delete_branch(),
        _ => Ok(()),
    }
}

fn switch_branch() -> Result<()> {
    let branches = get_branches()?;
    if branches.is_empty() {
        return Err(anyhow::anyhow!("No branches found"));
    }

    let current = get_current_branch()?;
    let default_index = branches.iter().position(|b| b == &current).unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch to switch to")
        .default(default_index)
        .items(&branches)
        .interact()
        .context("Failed to select branch")?;

    let target = &branches[selection];

    if target == &current {
        println!("Already on branch '{}'", target);
        return Ok(());
    }

    let status = Command::new("git")
        .args(&["checkout", target])
        .status()
        .context("Failed to switch branch")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to checkout branch {}", target));
    }

    Ok(())
}

fn create_branch() -> Result<()> {
    let types = vec!["feat", "fix", "chore", "docs", "refactor", "style", "test", "other"];
    
    let type_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch type")
        .default(0)
        .items(&types)
        .interact()
        .context("Failed to select branch type")?;

    let prefix = types[type_selection];

    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Branch name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Branch name cannot be empty")
            } else if input.contains(char::is_whitespace) {
                Err("Branch name cannot contain spaces")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to read branch name")?;

    let full_name = if prefix == "other" {
        name
    } else {
        format!("{}/{}", prefix, name)
    };

    let status = Command::new("git")
        .args(&["checkout", "-b", &full_name])
        .status()
        .context("Failed to create branch")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create branch {}", full_name));
    }

    Ok(())
}

fn delete_branch() -> Result<()> {
    let current = get_current_branch()?;
    let branches = get_branches()?;
    
    let available_to_delete: Vec<String> = branches
        .into_iter()
        .filter(|b| b != &current)
        .collect();

    if available_to_delete.is_empty() {
        println!("No other branches available to delete.");
        return Ok(());
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branches to delete (SPACE to select, ENTER to confirm)")
        .items(&available_to_delete)
        .interact()
        .context("Failed to select branches")?;

    if selections.is_empty() {
        println!("No branches selected.");
        return Ok(());
    }

    for index in selections {
        let branch_name = &available_to_delete[index];
        let status = Command::new("git")
            .args(&["branch", "-D", branch_name])
            .status()
            .context("Failed to execute git branch -D")?;

        if status.success() {
            println!("Deleted branch '{}'", branch_name);
        } else {
            eprintln!("Failed to delete branch '{}'", branch_name);
        }
    }

    Ok(())
}

fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["branch", "--format=%(refname:short)"])
        .output()
        .context("Failed to list branches")?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}