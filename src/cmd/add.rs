use anyhow::{Context, Result};
use std::process::Command;

pub fn run(files: Vec<String>) -> Result<()> {
    let mut args = vec!["add"];
    let refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(refs);

    if args.len() == 1 {
        args.push(".");
    }

    let status = Command::new("git")
        .args(&args)
        .status()
        .context("Failed to execute git add")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git add failed"));
    }

    Ok(())
}