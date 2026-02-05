use anyhow::{Context, Result};
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("Local changes detected. Stashing...");
        stash_push()?;
        stashed = true;
    }

    println!("Fetching and rebasing...");
    if let Err(_) = pull_rebase() {
        handle_rebase_conflict_loop()?;
    }

    if stashed {
        println!("Restoring local changes...");
        if let Err(_) = stash_pop() {
            handle_stash_conflict_loop()?;
        }
    }

    println!("Sync completed successfully.");
    Ok(())
}

fn check_if_dirty() -> Result<bool> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    Ok(!output.stdout.is_empty())
}

fn stash_push() -> Result<()> {
    let status = Command::new("git")
        .args(&["stash", "push", "-m", "wally-auto-sync"])
        .status()
        .context("Failed to stash changes")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to execute git stash"));
    }
    Ok(())
}

fn stash_pop() -> Result<()> {
    let status = Command::new("git")
        .args(&["stash", "pop"])
        .status()
        .context("Failed to pop stash")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Stash pop failed"));
    }
    Ok(())
}

fn pull_rebase() -> Result<()> {
    let status = Command::new("git")
        .args(&["pull", "--rebase"])
        .status()
        .context("Failed to execute git pull --rebase")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Rebase failed"));
    }
    Ok(())
}

fn handle_rebase_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING REBASE"));
        println!("The following files have merge conflicts:\n");

        let conflicted_files = get_conflicted_files()?;
        if conflicted_files.is_empty() {
            println!("{}", green.apply_to("No conflicted files found."));
        } else {
            for file in &conflicted_files {
                print_conflict_details(file)?;
            }
        }

        println!("\n{}", yellow.apply_to("Please open the files above, resolve the conflicts, and save them."));

        let choices = vec!["I have resolved the conflicts (Continue)", "Abort Sync"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            Command::new("git").args(&["rebase", "--abort"]).status()?;
            return Err(anyhow::anyhow!("Sync aborted by user"));
        }

        println!("Staging changes...");
        Command::new("git").args(&["add", "."]).status()?;

        println!("Continuing rebase...");
        let status = Command::new("git")
            .env("GIT_EDITOR", "true") 
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("{}", green.apply_to("Rebase resolved successfully!"));
            break;
        } else {
            println!("{}", red.apply_to("Rebase continue failed. Conflicts might still exist."));
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    Ok(())
}

fn handle_stash_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING STASH POP"));
        println!("Your local changes conflict with the incoming updates.\n");

        let conflicted_files = get_conflicted_files()?;
        for file in &conflicted_files {
            print_conflict_details(file)?;
        }

        println!("\n{}", yellow.apply_to("Please resolve the conflicts in the files above."));

        let choices = vec!["I have resolved the conflicts", "Abort (Changes remain in stash list)"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            return Err(anyhow::anyhow!("Stash pop cleanup aborted. You may need to reset or drop stash manually."));
        }

        let remaining_conflicts = get_conflicted_files()?;
        if remaining_conflicts.is_empty() {
            println!("{}", green.apply_to("Conflicts resolved."));
            Command::new("git").args(&["stash", "drop"]).status()?;
            break;
        } else {
            println!("{}", red.apply_to("Conflicts still detected. Please ensure markers are removed."));
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    Ok(())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    let mut files = Vec::new();

    for line in stdout.lines() {
        // Look for 'UU', 'AA', 'UD', etc.
        if line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DU") || line.starts_with("UD") {
            if line.len() > 3 {
                files.push(line[3..].to_string());
            }
        }
    }

    Ok(files)
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan();
    let blue = Style::new().blue();
    
    println!("{}", cyan.apply_to(format!("File: {}", file_path)));
    
    if !path.exists() {
        return Ok(());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    let mut inside_conflict = false;
    let mut printed_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("<<<<<<<") {
            inside_conflict = true;
            println!("  {}", blue.apply_to(format!("Line {}: Start of conflict", i + 1)));
        }

        if inside_conflict {
            println!("    {}", line);
        }

        if line.starts_with(">>>>>>>") {
            inside_conflict = false;
            println!("  {}", blue.apply_to(format!("Line {}: End of conflict", i + 1)));
            println!("");
            printed_count += 1;
            if printed_count >= 3 {
                println!("    ... (more conflicts hidden) ...");
                break;
            }
        }
    }

    Ok(())
}