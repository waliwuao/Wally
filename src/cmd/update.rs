use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select, Confirm};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run() -> Result<()> {
    let success = Style::new().green().bold();
    let warning = Style::new().yellow();
    let dim = Style::new().dim();

    // 1. 分支检查与重命名建议
    println!("{} Checking branch...", dim.apply_to("[1/4]"));
    let current_branch = get_current_branch()?;
    if current_branch == "master" {
        println!("{}", warning.apply_to("   Current branch is 'master'."));
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("   Rename to 'main' to follow modern standards?")
            .default(true)
            .interact()? 
        {
            execute_git(&["branch", "-m", "master", "main"])?;
        }
    }

    // 2. 检查暂存区
    println!("{} Checking workspace...", dim.apply_to("[2/4]"));
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("   {}", warning.apply_to("Uncommitted changes found. Stashing safely..."));
        execute_git(&["stash", "push", "-m", "wally-auto-update"])?;
        stashed = true;
    }

    // 3. 拉取并变基
    println!("{} Pulling from remote...", dim.apply_to("[3/4]"));
    let pull_status = execute_git(&["pull", "--rebase"])?;

    if !pull_status.success() {
        if is_rebase_in_progress()? {
            handle_rebase_conflict_loop()?;
        } else {
            let remote = "origin";
            let branch = get_current_branch()?;
            println!("   Standard pull failed. Trying {}/{}...", remote, branch);
            let retry_status = execute_git(&["pull", "--rebase", remote, &branch])?;
            if !retry_status.success() && is_rebase_in_progress()? {
                handle_rebase_conflict_loop()?;
            }
        }
    }

    // 4. 恢复暂存区
    println!("{} Finalizing...", dim.apply_to("[4/4]"));
    if stashed {
        println!("   Restoring your stashed changes...");
        let status = execute_git(&["stash", "pop"])?;
        if !status.success() {
            println!("{}", Style::new().red().bold().apply_to("   STASH CONFLICT!"));
            println!("   Your changes collided with remote changes. Please fix markers manually.");
        }
    }

    println!("\n{}", success.apply_to("Update successful!"));
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
    
    loop {
        let files = get_conflicted_files()?;
        if files.is_empty() { break; }

        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING UPDATE"));
        
        for file in &files {
            print_conflict_details(file)?;
        }

        let choices = vec!["I have resolved the conflicts", "Abort update"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Action required")
            .items(&choices)
            .default(0)
            .interact()?;

        if selection == 1 {
            execute_git(&["rebase", "--abort"])?;
            return Err(anyhow::anyhow!("Update aborted by user."));
        }

        execute_git(&["add", "."])?;
        // 使用环境变量避免弹出编辑器，直接尝试继续
        let status = std::process::Command::new("git")
            .env("GIT_EDITOR", "true") 
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("Conflicts resolved and update continued.");
            break;
        } else {
            println!("{}", red.apply_to("Still have conflicts. Please check the markers in files."));
            thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(())
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan().bold();
    println!("\n{}", cyan.apply_to(format!("File: {}", file_path)));
    
    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut inside_conflict = false;
        
        for line_res in reader.lines().take(50) { // 最多展示50行以防刷屏
            let line = line_res.unwrap_or_default();
            if line.starts_with("<<<<<<<") { inside_conflict = true; }
            if inside_conflict {
                println!("    {}", line);
            }
            if line.starts_with(">>>>>>>") { inside_conflict = false; }
        }
    }
    Ok(())
}