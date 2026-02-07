use crate::cmd::{execute_git, execute_git_output, get_remotes, add_remote_workflow};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select, MultiSelect};

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
    let output = execute_git_output(&["diff", "--cached", "--name-only"])?;
    let staged = String::from_utf8(output.stdout)?;

    if staged.trim().is_empty() {
        println!("Nothing staged to commit.");
        println!("Please run 'wally add' first to select files.");
        return Ok(());
    }

    let message = build_commit_message()?;
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
        .with_prompt("Select change type")
        .default(0)
        .items(&items)
        .clear(true)
        .interact()?;

    let selected_type = COMMIT_TYPES[selection].code;

    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact()?;

    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() { Err("Subject cannot be empty") } else { Ok(()) }
        })
        .interact()?;

    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (optional)")
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
    let branch_output = execute_git_output(&["branch", "--show-current"])?;
    let current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    let mut remotes = get_remotes()?;

    if remotes.is_empty() {
        let new_remote = add_remote_workflow()?;
        remotes.push(new_remote);
    } else if remotes.len() > 1 {
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select remotes to push to")
            .items(&remotes)
            .defaults(&vec![true; remotes.len()])
            .interact()?;
        
        if selections.is_empty() {
            println!("No remotes selected.");
            return Ok(());
        }
        remotes = selections.iter().map(|&i| remotes[i].clone()).collect();
    }

    for remote in remotes {
        println!("Pushing to {}...", remote);
        let status = execute_git(&["push", &remote, &current_branch])?;
        if !status.success() {
            execute_git(&["push", "-u", &remote, &current_branch])?;
        }
    }
    Ok(())
}