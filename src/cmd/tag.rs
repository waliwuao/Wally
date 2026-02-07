use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};

pub fn run() -> Result<()> {
    let yellow = Style::new().yellow();
    
    let current_tag = get_latest_tag()?;
    let (major, minor, patch) = parse_version(&current_tag);

    println!("Current latest tag: {}", yellow.apply_to(&current_tag));

    let options = vec![
        format!("Patch: v{}.{}.{} (Bug fixes)", major, minor, patch + 1),
        format!("Minor: v{}.{}.0 (New features)", major, minor + 1),
        format!("Major: v{}.0.0 (Breaking changes)", major + 1),
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select next version type")
        .default(0)
        .items(&options)
        .clear(true)
        .interact()?;

    let next_tag = match selection {
        0 => format!("v{}.{}.{}", major, minor, patch + 1),
        1 => format!("v{}.{}.0", major, minor + 1),
        2 => format!("v{}.0.0", major + 1),
        _ => unreachable!(),
    };

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Create and push tag '{}'?", next_tag))
        .default(true)
        .interact()?
    {
        create_and_push_tag(&next_tag)?;
    }

    Ok(())
}

fn get_latest_tag() -> Result<String> {
    let output = execute_git_output(&["describe", "--tags", "--abbrev=0"])?;
    
    if !output.status.success() {
        return Ok("v0.0.0".to_string());
    }

    let tag = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(if tag.is_empty() { "v0.0.0".to_string() } else { tag })
}

fn parse_version(tag: &str) -> (u32, u32, u32) {
    let cleaned = tag.trim_start_matches('v');
    let parts: Vec<u32> = cleaned
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    match parts.len() {
        3 => (parts[0], parts[1], parts[2]),
        2 => (parts[0], parts[1], 0),
        1 => (parts[0], 0, 0),
        _ => (0, 0, 0),
    }
}

fn create_and_push_tag(tag: &str) -> Result<()> {
    let status = execute_git(&["tag", "-a", tag, "-m", &format!("Release {}", tag)])?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create local tag"));
    }

    println!("Local tag '{}' created.", tag);

    let push_status = execute_git(&["push", "origin", tag])?;

    if push_status.success() {
        println!("Successfully pushed tag '{}' to origin.", tag);
    } else {
        println!("Warning: Tag created but failed to push to origin. Check your remote settings.");
    }

    Ok(())
}