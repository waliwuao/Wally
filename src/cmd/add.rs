use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use console::{Key, Style, Term};

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
    let help_style = Style::new().dim();
    
    const MAX_LIST_HEIGHT: usize = 10; 

    loop {
        term.clear_screen()?;
        
        let total = entries.len();
        let (start_idx, end_idx) = if total <= MAX_LIST_HEIGHT {
            (0, total)
        } else {
            let half = MAX_LIST_HEIGHT / 2;
            if cursor < half {
                (0, MAX_LIST_HEIGHT)
            } else if cursor + half >= total {
                (total - MAX_LIST_HEIGHT, total)
            } else {
                (cursor - half, cursor - half + MAX_LIST_HEIGHT)
            }
        };

        println!("{}", help_style.apply_to("[↑/↓] Move | [SPACE] Toggle | [a] All | [→] Diff | [←] Hide | [ENTER] Done"));
        if start_idx > 0 {
            println!("{}", help_style.apply_to("  ..."));
        }

        for i in start_idx..end_idx {
            let entry = &entries[i];
            let is_cursor = i == cursor;
            
            let checkbox = if entry.selected { 
                Style::new().green().apply_to("✔") 
            } else { 
                Style::new().dim().apply_to("○") 
            };
            
            let indicator = if is_cursor { 
                Style::new().cyan().bold().apply_to(">") 
            } else { 
                Style::new().apply_to(" ") 
            };
            
            let status_style = match entry.status.trim() {
                "M" => Style::new().yellow(),
                "A" | "??" => Style::new().green(),
                "D" => Style::new().red(),
                _ => Style::new().cyan(),
            };

            let path_style = if is_cursor { Style::new().bold() } else { Style::new() };
            
            println!("{} {} {} {}", 
                indicator,
                checkbox,
                status_style.apply_to(&entry.status),
                path_style.apply_to(&entry.path)
            );

            if entry.expanded {
                show_full_diff(&entry.path)?;
            }
        }

        if end_idx < total {
            println!("{}", help_style.apply_to("  ..."));
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
            Key::Char('a') => {
                // Toggle all: if all selected -> deselect all, otherwise select all
                let all_selected = entries.iter().all(|e| e.selected);
                for entry in &mut entries {
                    entry.selected = !all_selected;
                }
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

fn show_full_diff(path: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(&["diff", "--color=always", path])
        .output()
        .context("Failed to get diff")?;
    
    let content = String::from_utf8_lossy(&output.stdout);
    
    println!("{}", Style::new().dim().apply_to("  --------------------------------------------------"));
    
    if content.trim().is_empty() {
        println!("      {}", Style::new().dim().apply_to("(New file or no text diff available)"));
    } else {
        for line in content.lines() {
            println!("      {}", line);
        }
    }
    println!("{}", Style::new().dim().apply_to("  --------------------------------------------------"));
    Ok(())
}