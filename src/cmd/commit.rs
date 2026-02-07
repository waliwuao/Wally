use crate::cmd::{execute_git, execute_git_output, print_step};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

struct CommitType<'a> {
    code: &'a str,
    desc: &'a str,
}

const COMMIT_TYPES: &[CommitType] = &[
    CommitType { code: "feat", desc: "A new feature" },
    CommitType { code: "fix", desc: "A bug fix" },
    CommitType { code: "docs", desc: "Documentation only changes" },
    CommitType { code: "style", desc: "Changes that do not affect the meaning of the code" },
    CommitType { code: "refactor", desc: "Code change that neither fixes a bug nor adds feature" },
    CommitType { code: "perf", desc: "A code change that improves performance" },
    CommitType { code: "test", desc: "Adding missing tests or correcting existing tests" },
    CommitType { code: "build", desc: "Changes that affect the build system" },
    CommitType { code: "ci", desc: "Changes to our CI configuration files" },
    CommitType { code: "chore", desc: "Other changes that don't modify src or test files" },
    CommitType { code: "revert", desc: "Reverts a previous commit" },
];

pub fn run() -> Result<()> {
    print_step("Checking Staged Files");

    // Check if anything is staged
    let output = execute_git_output(&["diff", "--cached", "--name-only"])?;
    let staged = String::from_utf8(output.stdout)?;

    if staged.trim().is_empty() {
        println!("Nothing staged to commit.");
        println!("Please run 'wally add' first to select files.");
        return Ok(());
    }

    let message = build_commit_message()?;
    
    print_step("Committing");
    let status = execute_git(&["commit", "-m", &message])?;

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
    print_step("Pushing to Remote");
    
    let branch_output = execute_git_output(&["branch", "--show-current"])?;
    let mut current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    if current_branch == "master" {
        println!("Detected 'master' branch. Renaming to 'main'...");
        execute_git(&["branch", "-m", "master", "main"])?;
        current_branch = "main".to_string();
    }

    let remote_output = execute_git_output(&["remote"])?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL to add origin")
            .interact_text()?;
        
        execute_git(&["remote", "add", "origin", &url])?;
        execute_git(&["push", "-u", "origin", &current_branch])?;
    } else {
        let status = execute_git(&["push"])?;
        if !status.success() {
            println!("Standard push failed. Trying to set upstream...");
            execute_git(&["push", "-u", "origin", &current_branch])?;
        }
    }
    Ok(())
}