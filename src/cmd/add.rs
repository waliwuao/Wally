use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use console::{Key, Style, Term};

struct FileEntry {
    path: String,
    status: String,
    selected: bool,
    expanded: bool,
    diff_cache: Vec<String>,
}

#[derive(Clone)]
enum RenderLine {
    // 存储该行所属的文件在 entries 里的索引
    FileHeader(usize),
    // 存储该行所属的文件索引，以及该行在 diff_cache 里的行索引
    DiffLine(usize, usize),
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
                diff_cache: Vec::new(),
            });
        }
    }

    let term = Term::stdout();
    let mut cursor_line = 0; // 现在光标指向的是“行”，而不是“文件”
    let mut viewport_top = 0;
    let help_style = Style::new().dim();
    
    loop {
        let (term_height, _) = term.size();
        let display_height = (term_height as usize).saturating_sub(4); 

        // 1. 构建当前所有可见的逻辑行
        let mut all_lines: Vec<RenderLine> = Vec::new();
        for (f_idx, entry) in entries.iter().enumerate() {
            all_lines.push(RenderLine::FileHeader(f_idx));
            if entry.expanded {
                for d_idx in 0..entry.diff_cache.len() {
                    all_lines.push(RenderLine::DiffLine(f_idx, d_idx));
                }
            }
        }

        // 修正光标边界，防止收起 Diff 时光标悬空
        if cursor_line >= all_lines.len() {
            cursor_line = all_lines.len().saturating_sub(1);
        }

        // 2. 视口滚动逻辑：确保光标所在行始终在屏幕内
        if cursor_line < viewport_top {
            viewport_top = cursor_line;
        } else if cursor_line >= viewport_top + display_height {
            viewport_top = cursor_line - display_height + 1;
        }

        // 3. 渲染
        term.clear_screen()?;
        println!("{}", help_style.apply_to("[↑/↓] Move Line | [SPACE] Toggle | [a] All | [→] Expand | [←] Hide | [ENTER] Done"));
        println!("{}", help_style.apply_to("-----------------------------------------------------------------------"));

        let end_idx = (viewport_top + display_height).min(all_lines.len());
        for i in viewport_top..end_idx {
            let is_cursor = i == cursor_line;
            let indicator = if is_cursor { Style::new().cyan().bold().apply_to(">") } else { Style::new().apply_to(" ") };

            match all_lines[i] {
                RenderLine::FileHeader(f_idx) => {
                    let entry = &entries[f_idx];
                    let checkbox = if entry.selected { Style::new().green().apply_to("✔") } else { Style::new().dim().apply_to("○") };
                    let status_style = match entry.status.trim() {
                        "M" => Style::new().yellow(),
                        "A" | "??" => Style::new().green(),
                        "D" => Style::new().red(),
                        _ => Style::new().cyan(),
                    };
                    let path_style = if is_cursor { Style::new().bold().underlined() } else { Style::new() };
                    
                    println!("{} {} {:<2} {}", indicator, checkbox, status_style.apply_to(&entry.status), path_style.apply_to(&entry.path));
                },
                RenderLine::DiffLine(f_idx, d_idx) => {
                    let diff_text = &entries[f_idx].diff_cache[d_idx];
                    if is_cursor {
                        println!("{}     {}", indicator, diff_text);
                    } else {
                        println!("      {}", diff_text);
                    }
                }
            }
        }

        // 4. 交互处理
        let key = term.read_key()?;
        match key {
            Key::ArrowUp => {
                if cursor_line > 0 { cursor_line -= 1; }
            },
            Key::ArrowDown => {
                if cursor_line < all_lines.len() - 1 { cursor_line += 1; }
            },
            Key::Char(' ') => {
                let f_idx = match all_lines[cursor_line] {
                    RenderLine::FileHeader(idx) => idx,
                    RenderLine::DiffLine(idx, _) => idx,
                };
                entries[f_idx].selected = !entries[f_idx].selected;
            },
            Key::Char('a') => {
                let all_selected = entries.iter().all(|e| e.selected);
                for entry in &mut entries {
                    entry.selected = !all_selected;
                }
            },
            Key::ArrowRight => {
                if let RenderLine::FileHeader(f_idx) = all_lines[cursor_line] {
                    if !entries[f_idx].expanded {
                        entries[f_idx].expanded = true;
                        if entries[f_idx].diff_cache.is_empty() {
                            entries[f_idx].diff_cache = fetch_diff(&entries[f_idx].path)?;
                        }
                    }
                }
            },
            Key::ArrowLeft => {
                if let RenderLine::FileHeader(f_idx) = all_lines[cursor_line] {
                    entries[f_idx].expanded = false;
                } else if let RenderLine::DiffLine(f_idx, _) = all_lines[cursor_line] {
                    entries[f_idx].expanded = false;
                }
            },
            Key::Enter => break,
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

fn fetch_diff(path: &str) -> Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(&["diff", "--color=always", path])
        .output()
        .context("Failed to get diff")?;
    
    let content = String::from_utf8_lossy(&output.stdout);
    if content.trim().is_empty() {
        Ok(vec![Style::new().dim().apply_to("(New file or no text diff available)").to_string()])
    } else {
        Ok(content.lines().map(|s| s.to_string()).collect())
    }
}