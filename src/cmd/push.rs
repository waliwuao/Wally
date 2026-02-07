use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

pub fn run() -> Result<()> {
    // 1. Get current branch and handle master->main rename
    let branch_output = execute_git_output(&["branch", "--show-current"])?;
    let mut current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    if current_branch == "master" {
        println!("Detected 'master' branch. Renaming to 'main'...");
        execute_git(&["branch", "-m", "master", "main"])?;
        current_branch = "main".to_string();
    }

    // 2. Check if remote exists
    let remote_output = execute_git_output(&["remote"])?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL")
            .interact_text()?;
        
        execute_git(&["remote", "add", "origin", &url])?;
        execute_git(&["push", "-u", "origin", &current_branch])?;
    } else {
        // 3. Try standard push
        let status = execute_git(&["push"])?;
        
        if !status.success() {
            let warning = Style::new().yellow();
            println!("{}", warning.apply_to("Push failed. Checking reasons..."));

            // Check if upstream is missing
            let upstream_check = execute_git_output(&["rev-parse", "--abbrev-ref", "@{u}"]);
            if upstream_check.is_err() || !upstream_check.unwrap().status.success() {
                 println!("Setting upstream to origin/{}...", current_branch);
                 let set_upstream_status = execute_git(&["push", "-u", "origin", &current_branch])?;
                 if set_upstream_status.success() {
                     println!("{}", Style::new().green().apply_to("Push successful!"));
                 }
                 return Ok(());
            }

            // If upstream exists but push failed, check for divergence (Squash/Amend)
            let confirm_force = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Remote history differs (likely due to Squash/Amend). Force push?")
                .default(false)
                .interact()?;

            if confirm_force {
                println!("Executing force push (safe lease)...");
                let force_status = execute_git(&["push", "--force-with-lease"])?;
                if !force_status.success() {
                    println!("{}", Style::new().red().apply_to("Force push failed. Someone else may have pushed changes."));
                } else {
                    println!("{}", Style::new().green().apply_to("Force push successful."));
                }
            } else {
                println!("Push aborted. You may need to 'git pull' manually.");
            }
        } else {
            println!("{}", Style::new().green().apply_to("Push successful!"));
        }
    }
    Ok(())
}