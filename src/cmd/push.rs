use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

pub fn run() -> Result<()> {
    // 1. Get current branch and handle master->main with CONFIRMATION
    let branch_output = execute_git_output(&["branch", "--show-current"])?;
    let mut current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    if current_branch == "master" {
        let yellow = Style::new().yellow();
        println!("{}", yellow.apply_to("Detected 'master' branch. The modern standard is 'main'."));
        
        let confirm_rename = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to rename your local 'master' branch to 'main'?")
            .default(true)
            .interact()?;

        if confirm_rename {
            execute_git(&["branch", "-m", "master", "main"])?;
            current_branch = "main".to_string();
            println!("{}", Style::new().green().apply_to("Branch renamed to 'main'."));
        }
    }

    // 2. Check if remote exists
    let remote_output = execute_git_output(&["remote"])?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL (e.g., https://github.com/user/repo.git)")
            .interact_text()?;
        
        execute_git(&["remote", "add", "origin", &url])?;
        println!("Setting upstream to origin/{}...", current_branch);
        execute_git(&["push", "-u", "origin", &current_branch])?;
    } else {
        // 3. Try standard push
        let status = execute_git(&["push"])?;
        
        if !status.success() {
            let warning = Style::new().yellow();
            println!("{}", warning.apply_to("\nPush failed. Analyzing reason..."));

            // Check if upstream is missing
            let upstream_check = execute_git_output(&["rev-parse", "--abbrev-ref", "@{u}"]);
            if upstream_check.is_err() || !upstream_check.unwrap().status.success() {
                 println!("No upstream branch set. Setting to origin/{}...", current_branch);
                 execute_git(&["push", "-u", "origin", &current_branch])?;
                 return Ok(());
            }

            // Conflict / Divergence
            let confirm_force = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Remote history differs. This happens after Squash/Reset. Force push?")
                .default(false)
                .interact()?;

            if confirm_force {
                println!("Executing safe force push...");
                let force_status = execute_git(&["push", "--force-with-lease"])?;
                if !force_status.success() {
                    println!("{}", Style::new().red().apply_to("Force push failed. Someone else may have pushed new changes."));
                }
            } else {
                println!("Push aborted. You should 'wally update' to sync with remote.");
            }
        } else {
            println!("{}", Style::new().green().apply_to("Push successful!"));
        }
    }
    Ok(())
}