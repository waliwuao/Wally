use crate::cmd::{execute_git, execute_git_output, print_step};
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run() -> Result<()> {
    let success = Style::new().green().bold();
    let warning = Style::new().yellow();
    let dim = Style::new().dim();

    print_step("Starting Update Process");

    println!("{} Checking branch name...", dim.apply_to("[1/4]"));
    let current_branch = get_current_branch()?;
    if current_branch == "master" {
        println!("   {}", warning.apply_to("Renaming 'master' to 'main' for compatibility..."));
        execute_git(&["branch", "-m", "master", "main"])?;
    }

    println!("{} Checking workspace status...", dim.apply_to("[2/4]"));
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("   {}", warning.apply_to("Uncommitted changes found. Stashing locally..."));
        execute_git(&["stash", "push", "-m", "wally-auto-update"])?;
        stashed = true;
    }

    println!("{} Pulling latest changes from remote...", dim.apply_to("[3/4]"));
    let pull_status = execute_git(&["pull", "--rebase"])?;

    if !pull_status.success() {
        if is_rebase_in_progress()? {
            handle_rebase_conflict_loop()?;
        } else {
            let remote = "origin";
            let branch = get_current_branch()?;
            println!("   {}", warning.apply_to(format!("Standard pull failed. Retrying with {}/{}...", remote, branch)));
            let retry_status = execute_git(&["pull", "--rebase", remote, &branch])?;
            if !retry_status.success() {
                if is_rebase_in_progress()? {
                    handle_rebase_conflict_loop()?;
                } else {
                    return Err(anyhow::anyhow!("Update failed. Please check network or remote settings."));
                }
            }
        }
    }

    println!("{} Finalizing workspace...", dim.apply_to("[4/4]"));
    if stashed {
        println!("   {}", warning.apply_to("Restoring your stashed changes..."));
        let status = execute_git(&["stash", "pop"])?;
        if !status.success() {
            handle_stash_conflict_loop()?;
        }
    }

    println!("\n{}", success.apply_to("Update completed successfully!"));
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = execute_git_output(&["branch", "--show-current"])?;
    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(if branch.is_empty() { "main".to_string() } else { branch })
}

fn check_if_dirty() -> Result<bool> {
    let output = execute_git_output(&["status", "--porcelain"])?;
    Ok(!output.stdout.is_empty())
}

fn is_rebase_in_progress() -> Result<bool> {
    let output = execute_git_output(&["rev-parse", "--git-dir"])?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    let path = Path::new(&git_dir);
    Ok(path.join("rebase-merge").exists() || path.join("rebase-apply").exists())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = execute_git_output(&["status", "--porcelain"])?;
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
        println!("{}", red.apply_to("CONFLICTS DETECTED"));
        println!("Please resolve conflicts in these files:\n");

        for file in &files {
            print_conflict_details(file)?;
        }

        println!("{}", yellow.apply_to("How to resolve:"));
        println!("1. Open files, look for markers, and keep the desired code.");
        println!("2. Save files and return here.\n");

        let choices = vec!["I have resolved all conflicts", "Abort Update"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            execute_git(&["rebase", "--abort"])?;
            return Err(anyhow::anyhow!("Update aborted."));
        }

        execute_git(&["add", "."])?;
        // For rebase --continue, we need to handle editor invocation if it happens
        let status = std::process::Command::new("git")
            .env("GIT_EDITOR", "true")
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("{}", green.apply_to("Rebase continued successfully!"));
            break;
        } else {
            println!("{}", red.apply_to("Conflicts still exist. Please check again."));
            thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(())
}

fn handle_stash_conflict_loop() -> Result<()> {
    let red = Style::new().red().bold();
    println!("\n{}", red.apply_to("STASH POP CONFLICT"));
    let files = get_conflicted_files()?;
    for file in &files { println!("  - {}", file); }
    println!("\nPlease resolve markers manually. Your work is safe in 'git stash list'.");
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
        for (i, line_res) in reader.lines().enumerate() {
            let line = line_res.unwrap_or_else(|_| String::new());
            if line.starts_with("<<<<<<<") { 
                inside = true; 
                println!("  {}", blue.apply_to(format!("Line {}:", i + 1))); 
            }
            if inside { 
                println!("    {}", line); 
            }
            if line.starts_with(">>>>>>>") { 
                break; 
            }
        }
    }
    println!();
    Ok(())
}