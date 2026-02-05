use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::process::Command;

struct CommitType<'a> {
    code: &'a str,
    desc: &'a str,
}

const COMMIT_TYPES: &[CommitType] = &[
    CommitType { code: "feat", desc: "A new feature" },
    CommitType { code: "fix", desc: "A bug fix" },
    CommitType { code: "docs", desc: "Documentation only changes" },
    CommitType { code: "style", desc: "Changes that do not affect the meaning of the code" },
    CommitType { code: "refactor", desc: "A code change that neither fixes a bug nor adds a feature" },
    CommitType { code: "perf", desc: "A code change that improves performance" },
    CommitType { code: "test", desc: "Adding missing tests or correcting existing tests" },
    CommitType { code: "build", desc: "Changes that affect the build system or external dependencies" },
    CommitType { code: "ci", desc: "Changes to our CI configuration files and scripts" },
    CommitType { code: "chore", desc: "Other changes that don't modify src or test files" },
    CommitType { code: "revert", desc: "Reverts a previous commit" },
];

pub fn run() -> Result<()> {
    if !stage_files()? {
        println!("No files staged. Skipping commit.");
        return Ok(());
    }

    let message = build_commit_message()?;
    
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .status()
        .context("Failed to execute git commit")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git commit failed"));
    }

    println!("Commit successful!");

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to push to remote?")
        .default(true)
        .interact()?
    {
        push_workflow()?;
    }

    Ok(())
}

fn stage_files() -> Result<bool> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to get git status")?;

    let stdout = String::from_utf8(output.stdout)?;
    if stdout.is_empty() {
        return Ok(false);
    }

    let mut files = Vec::new();
    let mut items = vec!["[ALL] (Select this to stage ALL changes)".to_string()];

    for line in stdout.lines() {
        if line.len() > 3 {
            let status_code = &line[0..2];
            let file = &line[3..];
            
            let status_text = match status_code {
                "??" => "Untracked",
                " M" | "M " => "Modified ",
                " A" | "A " => "Added    ",
                " D" | "D " => "Deleted  ",
                " R" | "R " => "Renamed  ",
                "UU" => "Conflict ",
                _ => status_code.trim(),
            };

            files.push(file.to_string());
            items.push(format!("[{}] {}", status_text.trim(), file));
        }
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select files to stage (SPACE to select, ENTER to confirm)")
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        return Ok(false);
    }

    if selections.contains(&0) {
        Command::new("git").args(&["add", "."]).status()?;
    } else {
        for &index in &selections {
            let file_index = index - 1;
            Command::new("git").arg("add").arg(&files[file_index]).status()?;
        }
    }

    Ok(true)
}

fn build_commit_message() -> Result<String> {
    let items: Vec<String> = COMMIT_TYPES
        .iter()
        .map(|t| format!("{:<10} {}", t.code, t.desc))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the type of change")
        .default(0)
        .items(&items)
        .interact()?;

    let selected_type = COMMIT_TYPES[selection].code;

    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact()?;

    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject (short description)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() { Err("Subject cannot be empty") } else { Ok(()) }
        })
        .interact()?;

    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (detailed description, optional)")
        .allow_empty(true)
        .interact()?;

    let mut message = if scope.trim().is_empty() {
        format!("{}: {}", selected_type, subject)
    } else {
        format!("{}({}): {}", selected_type, scope, subject)
    };

    if !body.trim().is_empty() {
        message.push_str("\n\n");
        message.push_str(&body);
    }

    Ok(message)
}

fn push_workflow() -> Result<()> {
    let remote_output = Command::new("git").args(&["remote"]).output()?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL to add origin")
            .interact_text()?;
        
        Command::new("git").args(&["remote", "add", "origin", &url]).status()?;
        Command::new("git").args(&["push", "-u", "origin", "main"]).status()?;
    } else {
        let status = Command::new("git").arg("push").status()?;
        if !status.success() {
            println!("Standard push failed. Trying to set upstream...");
            let branch_output = Command::new("git").args(&["branch", "--show-current"]).output()?;
            let branch = String::from_utf8(branch_output.stdout)?.trim().to_string();
            Command::new("git").args(&["push", "-u", "origin", &branch]).status()?;
        }
    }
    Ok(())
}