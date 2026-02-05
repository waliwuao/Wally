use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    let current_branch = get_current_branch()?;
    
    if current_branch == "master" {
        println!("Detected 'master' branch. Renaming to 'main' for compatibility...");
        Command::new("git").args(&["branch", "-m", "master", "main"]).status()?;
    }

    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("Local changes detected. Stashing to keep workspace clean...");
        stash_push()?;
        stashed = true;
    }

    println!("Synchronizing with remote...");
    
    let pull_status = Command::new("git")
        .args(&["pull", "--rebase"])
        .status()?;

    if !pull_status.success() {
        if is_rebase_in_progress()? {
            handle_rebase_conflict_loop()?;
        } else {
            println!("Standard pull failed (perhaps no tracking info).");
            let remote = "origin";
            let branch = get_current_branch()?;
            
            println!("Attempting to sync with {}/{}...", remote, branch);
            let retry_status = Command::new("git")
                .args(&["pull", "--rebase", remote, &branch])
                .status()?;
                
            if !retry_status.success() {
                if is_rebase_in_progress()? {
                    handle_rebase_conflict_loop()?;
                } else {
                    return Err(anyhow::anyhow!("Could not sync with remote. Please check your internet or remote settings."));
                }
            }
        }
    }

    if stashed {
        println!("Restoring your local changes...");
        if let Err(_) = stash_pop() {
            handle_stash_conflict_loop()?;
        }
    }

    println!("{}", Style::new().green().bold().apply_to("Sync completed successfully!"));
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    if branch.is_empty() {
        Ok("main".to_string())
    } else {
        Ok(branch)
    }
}

fn check_if_dirty() -> Result<bool> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    Ok(!output.stdout.is_empty())
}

fn stash_push() -> Result<()> {
    Command::new("git")
        .args(&["stash", "push", "-m", "wally-auto-sync"])
        .status()?;
    Ok(())
}

fn stash_pop() -> Result<()> {
    let status = Command::new("git").args(&["stash", "pop"]).status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Stash pop conflict"));
    }
    Ok(())
}

fn is_rebase_in_progress() -> Result<bool> {
    let output = Command::new("git").args(&["rev-parse", "--git-dir"]).output()?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    let path = Path::new(&git_dir);
    Ok(path.join("rebase-merge").exists() || path.join("rebase-apply").exists())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git").args(&["status", "--porcelain"]).output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines()
        .filter(|l| l.starts_with("UU") || l.starts_with("AA") || l.starts_with("DU") || l.starts_with("UD"))
        .filter_map(|l| if l.len() > 3 { Some(l[3..].to_string()) } else { None })
        .collect())
}

fn handle_rebase_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red().bold();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        let files = get_conflicted_files()?;
        if files.is_empty() { break; }

        term.clear_screen()?;
        println!("{}", red.apply_to("=== CONFLICTS DETECTED ==="));
        println!("The following files have markers (<<<<<<<, =======, >>>>>>>):\n");

        for file in &files {
            print_conflict_details(file)?;
        }

        println!("{}", yellow.apply_to("Instructions:"));
        println!("1. Open the files listed above in your editor.");
        println!("2. Look for conflict markers and decide which code to keep.");
        println!("3. Delete the markers and save the files.");
        println!("4. Return here and select 'Resolved'.\n");

        let choices = vec!["I have resolved all conflicts", "Abort Sync"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            Command::new("git").args(&["rebase", "--abort"]).status()?;
            return Err(anyhow::anyhow!("Sync aborted."));
        }

        println!("Applying resolutions...");
        Command::new("git").args(&["add", "."]).status()?;
        
        let status = Command::new("git")
            .env("GIT_EDITOR", "true")
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("{}", green.apply_to("Rebase continued successfully!"));
            break;
        } else {
            println!("{}", red.apply_to("Some conflicts are still not resolved. Please check again."));
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }
    Ok(())
}

fn handle_stash_conflict_loop() -> Result<()> {
    let red = Style::new().red().bold();
    let yellow = Style::new().yellow();

    println!("\n{}", red.apply_to("=== STASH POP CONFLICT ==="));
    let files = get_conflicted_files()?;
    for file in &files {
        println!("  - {}", file);
    }
    println!("\n{}", yellow.apply_to("Your local uncommitted changes conflicted with the new remote code."));
    println!("Please resolve them manually. Your work is safe in 'git stash list'.");
    Ok(())
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan().bold();
    let blue = Style::new().blue();
    
    println!("{}", cyan.apply_to(format!("File: {}", file_path)));
    
    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut inside = false;
        let mut count = 0;

        for (i, line_res) in reader.lines().enumerate() {
            let line = line_res.unwrap_or_default();
            if line.starts_with("<<<<<<<") {
                inside = true;
                println!("  {}", blue.apply_to(format!("Line {}:", i + 1)));
            }
            if inside {
                println!("    {}", line);
            }
            if line.starts_with(">>>>>>>") {
                inside = false;
                count += 1;
                if count >= 2 { break; }
            }
        }
    }
    println!();
    Ok(())
}