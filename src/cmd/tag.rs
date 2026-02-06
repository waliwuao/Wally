use anyhow::{Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let header = Style::new().cyan().bold();
    let yellow = Style::new().yellow();
    
    println!("{}", header.apply_to("\n--- Git Semantic Tagging ---"));

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
    let output = Command::new("git")
        .args(&["describe", "--tags", "--abbrev=0"])
        .output()?;

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
    let status = Command::new("git")
        .args(&["tag", "-a", tag, "-m", &format!("Release {}", tag)])
        .status()
        .context("Failed to create git tag")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create local tag"));
    }

    println!("Local tag '{}' created.", tag);

    let push_status = Command::new("git")
        .args(&["push", "origin", tag])
        .status()
        .context("Failed to push tag to origin")?;

    if push_status.success() {
        println!("Successfully pushed tag '{}' to origin.", tag);
    } else {
        println!("Warning: Tag created but failed to push to origin. Check your remote settings.");
    }

    Ok(())
}