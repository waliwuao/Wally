use crate::cmd::{execute_git, execute_git_output, get_remotes, add_remote_workflow};
use anyhow::Result;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

pub fn run() -> Result<()> {
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

    let mut remotes = get_remotes()?;

    if remotes.is_empty() {
        println!("{}", Style::new().yellow().apply_to("No remotes found. You need to add one to push."));
        let new_remote = add_remote_workflow()?;
        remotes.push(new_remote);
    } else {
        let add_new = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a new remote before pushing?")
            .default(false)
            .interact()?;

        if add_new {
            let new_name = add_remote_workflow()?;
            remotes = get_remotes()?; 
            println!("Remote '{}' added.", new_name);
        }
    }

    let selected_remotes = if remotes.len() == 1 {
        remotes
    } else {
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select remotes to push (Space to select, Enter to confirm)")
            .items(&remotes)
            .defaults(&vec![true; remotes.len()])
            .interact()?;
        
        if selections.is_empty() {
            println!("No remotes selected. Aborting.");
            return Ok(());
        }
        selections.iter().map(|&i| remotes[i].clone()).collect()
    };

    for remote in selected_remotes {
        println!("\nPushing to {}...", Style::new().cyan().bold().apply_to(&remote));
        let status = execute_git(&["push", &remote, &current_branch])?;
        
        if !status.success() {
            let warning = Style::new().yellow();
            println!("{}", warning.apply_to(format!("Push to {} failed. Checking for upstream/divergence...", remote)));

            let upstream_check = execute_git_output(&["rev-parse", "--abbrev-ref", &format!("{}@{{u}}", current_branch)]);
            if upstream_check.is_err() || !upstream_check.unwrap().status.success() {
                 println!("Setting upstream to {}/{}...", remote, current_branch);
                 execute_git(&["push", "-u", &remote, &current_branch])?;
                 continue;
            }

            let confirm_force = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Remote '{}' history differs. Force push?", remote))
                .default(false)
                .interact()?;

            if confirm_force {
                execute_git(&["push", &remote, "--force-with-lease"])?;
            }
        } else {
            println!("{}", Style::new().green().apply_to(format!("Push to {} successful!", remote)));
        }
    }
    Ok(())
}