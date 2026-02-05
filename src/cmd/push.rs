use anyhow::{Context, Result};
use std::process::Command;

pub fn run(url: Option<String>) -> Result<()> {
    if let Some(remote_url) = url {
        let _ = Command::new("git")
            .args(&["remote", "add", "origin", &remote_url])
            .output();

        let _ = Command::new("git")
            .args(&["remote", "set-url", "origin", &remote_url])
            .output();

        Command::new("git")
            .args(&["branch", "-M", "main"])
            .status()
            .context("Failed to rename branch to main")?;

        let status = Command::new("git")
            .args(&["push", "-u", "origin", "main"])
            .status()
            .context("Failed to push to remote")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Push failed"));
        }
    } else {
        let status = Command::new("git")
            .arg("push")
            .status()
            .context("Failed to execute git push")?;

        if !status.success() {
            return Err(anyhow::anyhow!("git push failed"));
        }
    }

    Ok(())
}