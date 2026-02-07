use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use console::{Key, Style, Term};
use std::collections::HashSet;

struct FileEntry {
    path: String,
    status: String,
    selected: bool,
    expanded: bool,
}

pub fn run() -> Result<()> {
    let output = execute_git_output(&["status", "--porcelain"])?;
    let stdout = String::from_utf8(output.stdout)?;
    
    if stdout.is_empty() {
        println!("No changes to stage.");
        return Ok(());
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    for line in stdout.lines() {
        if line.len() > 3 {
            let status = &line[0..2];
            let path = &line[3..];
            entries.push(FileEntry {
                path: path.to_string(),
                status: status.to_string(),
                selected: false,
                expanded: false,
            });
        }
    }

    let term = Term::stdout();
    let mut cursor = 0;

    // UI Loop
    loop {
        term.clear_screen()?;
        println!("{}", Style::new().magenta().bold().apply_to("--- Interactive Add ---"));
        println!("Controls: [↑/↓] Move | [SPACE] Toggle | [→] View Diff | [←] Hide Diff | [ENTER] Confirm\n");

        for (i, entry) in entries.iter().enumerate() {
            let is_cursor = i == cursor;
            
            let checkbox = if entry.selected { "[x]" } else { "[ ]" };
            let indicator = if is_cursor { ">" } else { " " };
            
            let status_style = match entry.status.trim() {
                "M" => Style::new().yellow(),
                "A" | "??" => Style::new().green(),
                "D" => Style::new().red(),
                _ => Style::new().cyan(),
            };

            let line_style = if is_cursor { Style::new().bold() } else { Style::new() };
            
            println!("{} {} {} {}", 
                indicator,
                line_style.apply_to(checkbox),
                status_style.apply_to(&entry.status),
                line_style.apply_to(&entry.path)
            );

            if entry.expanded {
                show_diff(&entry.path)?;
            }
        }

        let key = term.read_key()?;
        match key {
            Key::ArrowUp => {
                if cursor > 0 { cursor -= 1; }
            },
            Key::ArrowDown => {
                if cursor < entries.len() - 1 { cursor += 1; }
            },
            Key::Char(' ') => {
                entries[cursor].selected = !entries[cursor].selected;
            },
            Key::ArrowRight => {
                entries[cursor].expanded = true;
            },
            Key::ArrowLeft => {
                entries[cursor].expanded = false;
            },
            Key::Enter => {
                break;
            },
            Key::Escape => {
                println!("Operation cancelled.");
                return Ok(());
            }
            _ => {}
        }
    }

    let selected_files: Vec<String> = entries
        .into_iter()
        .filter(|e| e.selected)
        .map(|e| e.path)
        .collect();

    if selected_files.is_empty() {
        println!("No files selected.");
        return Ok(());
    }

    let mut args = vec!["add"];
    for file in &selected_files {
        args.push(file);
    }

    execute_git(&args)?;
    println!("{}", Style::new().green().apply_to("Files staged successfully!"));

    Ok(())
}

fn show_diff(path: &str) -> Result<()> {
    // Check if it's untracked (??). Git diff won't show untracked easily without --no-index or adding it first intent-to-add
    // Simple workaround: justcat file if untracked, else git diff
    
    // Attempt standard diff with color
    let output = std::process::Command::new("git")
        .args(&["diff", "--color=always", path])
        .output()
        .context("Failed to get diff")?;
    
    let content = String::from_utf8_lossy(&output.stdout);
    
    if content.trim().is_empty() {
        // Might be untracked or new file
        println!("      (New file or no diff available)");
    } else {
        for line in content.lines() {
            println!("      {}", line);
        }
    }
    println!();
    Ok(())
}