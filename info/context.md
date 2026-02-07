# Project Context: wally

## File Structure

```text
wally/
├── .gitignore
├── Cargo.toml
├── LICENSE
├── src/
│   ├── cli.rs
│   ├── cmd/
│   │   ├── add.rs
│   │   ├── branch.rs
│   │   ├── commit.rs
│   │   ├── context.rs
│   │   ├── install.rs
│   │   ├── list.rs
│   │   ├── menu.rs
│   │   ├── mod.rs
│   │   ├── new.rs
│   │   ├── push.rs
│   │   ├── reset.rs
│   │   ├── stats.rs
│   │   ├── tag.rs
│   │   ├── uninstall.rs
│   │   └── update.rs
│   ├── main.rs
│   └── models.rs
└── templates/
    └── default.json

```

## File Contents

### .gitignore
```
/target
Cargo.lock
README.md
README_CN.md
info/

```

### Cargo.toml
```toml
[package]
name = "wally"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
ignore = "0.4"
dialoguer = "0.11"
console = "0.15"
```

### LICENSE
```
MIT License

Copyright (c) 2026 waliwuao

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

```

### src/cli.rs
```rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wally")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    New {
        project_name: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
    },
    Context,
    List,
    Add,
    Commit,
    Push,
    Branch,
    Update,
    Reset,
    Stats,
    Tag,
    Install {
        path: String,
    },
    Uninstall {
        template_name: String,
    },
}
```

### src/cmd/add.rs
```rs
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
```

### src/cmd/branch.rs
```rs
use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

pub fn run() -> Result<()> {
    let actions = vec![
        "Switch Branch",
        "Create Branch",
        "Merge  Branch",
        "Delete Branch",
        "Squash Commits",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch action")
        .default(0)
        .items(&actions)
        .clear(true)
        .interact()
        .context("Failed to read selection")?;

    match selection {
        0 => switch_branch(),
        1 => create_branch(),
        2 => merge_branch(),
        3 => delete_branch(),
        4 => squash_commits(),
        _ => Ok(()),
    }
}

fn switch_branch() -> Result<()> {
    let branches = get_branches()?;
    if branches.is_empty() {
        return Err(anyhow::anyhow!("No branches found"));
    }

    let current = get_current_branch()?;
    let default_index = branches.iter().position(|b| b == &current).unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch to switch to")
        .default(default_index)
        .items(&branches)
        .clear(true)
        .interact()?;

    let target = &branches[selection];

    if target == &current {
        println!("Already on branch '{}'", target);
        return Ok(());
    }

    let status = execute_git(&["checkout", target])?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to checkout branch {}", target));
    }

    Ok(())
}

fn create_branch() -> Result<()> {
    let types = vec![
        "feat", "fix", "chore", "docs", "refactor", "style", "test", "other",
    ];

    let type_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch type")
        .default(0)
        .items(&types)
        .clear(true)
        .interact()?;

    let prefix = types[type_selection];

    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Branch name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Branch name cannot be empty")
            } else if input.contains(char::is_whitespace) {
                Err("Branch name cannot contain spaces")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let full_name = if prefix == "other" {
        name
    } else {
        format!("{}/{}", prefix, name)
    };

    let status = execute_git(&["checkout", "-b", &full_name])?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create branch {}", full_name));
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Push '{}' to remote (origin)?", full_name))
        .default(true)
        .interact()?
    {
        execute_git(&["push", "-u", "origin", &full_name])?;
    }

    Ok(())
}

fn merge_branch() -> Result<()> {
    let current = get_current_branch()?;
    let branches = get_branches()?;

    let available_to_merge: Vec<String> = branches
        .into_iter()
        .filter(|b| b != &current)
        .collect();

    if available_to_merge.is_empty() {
        println!("No other branches available to merge.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select branch to merge INTO '{}'", current))
        .items(&available_to_merge)
        .clear(true)
        .interact()?;

    let branch_to_merge = &available_to_merge[selection];

    println!("Merging '{}' into '{}'...", branch_to_merge, current);

    let status = execute_git(&["merge", branch_to_merge])?;

    if status.success() {
        println!("Successfully merged '{}'", branch_to_merge);
    } else {
        return Err(anyhow::anyhow!(
            "Merge failed (likely due to conflicts). Please resolve conflicts manually."
        ));
    }

    Ok(())
}

fn squash_commits() -> Result<()> {
    let current = get_current_branch()?;
    let branches = get_branches()?;
    
    let base_branches: Vec<String> = branches
        .into_iter()
        .filter(|b| b == "main" || b == "master" || b == "develop")
        .collect();

    if base_branches.is_empty() {
        return Err(anyhow::anyhow!("No base branch (main/master/develop) found to squash against."));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select base branch to squash against")
        .items(&base_branches)
        .default(0)
        .clear(true)
        .interact()?;

    let base = &base_branches[selection];

    if current == *base {
        return Err(anyhow::anyhow!("Cannot squash on the base branch itself. Switch to a feature branch."));
    }

    let merge_base_out = execute_git_output(&["merge-base", base, &current])?;
    
    let merge_base = String::from_utf8(merge_base_out.stdout)?.trim().to_string();

    if merge_base.is_empty() {
        return Err(anyhow::anyhow!("Could not find a common ancestor with {}", base));
    }

    println!("Squashing all commits from {}... All changes will be staged.", merge_base);

    let status = execute_git(&["reset", "--soft", &merge_base])?;

    if status.success() {
        println!("\nSuccess! Commits squashed into staged changes.");
        println!("Run 'wally commit' next. You will likely need to FORCE PUSH (which is handled automatically).");
    }

    Ok(())
}

fn delete_branch() -> Result<()> {
    let current = get_current_branch()?;
    let branches = get_branches()?;

    let available_to_delete: Vec<String> = branches
        .into_iter()
        .filter(|b| b != &current)
        .collect();

    if available_to_delete.is_empty() {
        println!("No other branches available to delete.");
        return Ok(());
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branches to delete (SPACE to select)")
        .items(&available_to_delete)
        .clear(true)
        .interact()?;

    if selections.is_empty() {
        println!("No branches selected.");
        return Ok(());
    }

    let mut deleted_successfully = Vec::new();

    for &index in &selections {
        let branch_name = &available_to_delete[index];
        let status = execute_git(&["branch", "-D", branch_name])?;

        if status.success() {
            println!("Deleted branch '{}' locally.", branch_name);
            deleted_successfully.push(branch_name.clone());
        } else {
            eprintln!("Failed to delete branch '{}' locally.", branch_name);
        }
    }

    if !deleted_successfully.is_empty() {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Also delete from remote (origin)?")
            .default(false)
            .interact()?
        {
            for branch_name in deleted_successfully {
                let status = execute_git(&["push", "origin", "--delete", &branch_name])?;
                
                if status.success() {
                    println!("Deleted branch '{}' from remote.", branch_name);
                }
            }
        }
    }

    Ok(())
}

fn get_branches() -> Result<Vec<String>> {
    let output = execute_git_output(&["branch", "--format=%(refname:short)"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
}

fn get_current_branch() -> Result<String> {
    let output = execute_git_output(&["branch", "--show-current"])?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
```

### src/cmd/commit.rs
```rs
use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

struct CommitType<'a> {
    code: &'a str,
    desc: &'a str,
}

const COMMIT_TYPES: &[CommitType] = &[
    CommitType { code: "feat", desc: "A new feature" },
    CommitType { code: "fix", desc: "A bug fix" },
    CommitType { code: "docs", desc: "Documentation only changes" },
    CommitType { code: "style", desc: "Changes that do not affect the meaning of the code" },
    CommitType { code: "refactor", desc: "Code change that neither fixes a bug nor adds feature" },
    CommitType { code: "perf", desc: "A code change that improves performance" },
    CommitType { code: "test", desc: "Adding missing tests or correcting existing tests" },
    CommitType { code: "build", desc: "Changes that affect the build system" },
    CommitType { code: "ci", desc: "Changes to our CI configuration files" },
    CommitType { code: "chore", desc: "Other changes that don't modify src or test files" },
    CommitType { code: "revert", desc: "Reverts a previous commit" },
];

pub fn run() -> Result<()> {
    // Check if anything is staged
    let output = execute_git_output(&["diff", "--cached", "--name-only"])?;
    let staged = String::from_utf8(output.stdout)?;

    if staged.trim().is_empty() {
        println!("Nothing staged to commit.");
        println!("Please run 'wally add' first to select files.");
        return Ok(());
    }

    let message = build_commit_message()?;
    
    let status = execute_git(&["commit", "-m", &message])?;

    if !status.success() {
        return Err(anyhow::anyhow!("git commit failed"));
    }

    println!("Commit successful!");

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to push to remote?")
        .default(true)
        .interact()?
    {
        push_workflow()?;
    }

    Ok(())
}

fn build_commit_message() -> Result<String> {
    let items: Vec<String> = COMMIT_TYPES
        .iter()
        .map(|t| format!("{:<10} {}", t.code, t.desc))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select change type")
        .default(0)
        .items(&items)
        .clear(true) // Clean UI
        .interact()?;

    let selected_type = COMMIT_TYPES[selection].code;

    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact()?;

    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() { Err("Subject cannot be empty") } else { Ok(()) }
        })
        .interact()?;

    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (optional)")
        .allow_empty(true)
        .interact()?;

    let mut message = if scope.trim().is_empty() {
        format!("{}: {}", selected_type, subject)
    } else {
        format!("{}({}): {}", selected_type, scope, subject)
    };

    if !body.trim().is_empty() {
        message.push_str("\n\n");
        message.push_str(&body);
    }

    Ok(message)
}

fn push_workflow() -> Result<()> {
    let branch_output = execute_git_output(&["branch", "--show-current"])?;
    let mut current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    if current_branch == "master" {
        println!("Detected 'master' branch. Renaming to 'main'...");
        execute_git(&["branch", "-m", "master", "main"])?;
        current_branch = "main".to_string();
    }

    let remote_output = execute_git_output(&["remote"])?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL")
            .interact_text()?;
        
        execute_git(&["remote", "add", "origin", &url])?;
        execute_git(&["push", "-u", "origin", &current_branch])?;
    } else {
        // Try standard push first
        let status = execute_git(&["push"])?;
        
        if !status.success() {
            // Handle failures (No upstream or Diverged history/Squash)
            let warning = Style::new().yellow();
            println!("{}", warning.apply_to("Push failed. Checking reasons..."));

            // Check if upstream is missing
            let upstream_check = execute_git_output(&["rev-parse", "--abbrev-ref", "@{u}"]);
            if upstream_check.is_err() || !upstream_check.unwrap().status.success() {
                 println!("Setting upstream to origin/{}...", current_branch);
                 execute_git(&["push", "-u", "origin", &current_branch])?;
                 return Ok(());
            }

            // If we are here, upstream exists but push failed. Likely divergence (Squash).
            let confirm_force = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Remote history differs (likely due to Squash). Force push?")
                .default(false)
                .interact()?;

            if confirm_force {
                // Use force-with-lease for safety
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
        }
    }
    Ok(())
}
```

### src/cmd/context.rs
```rs
use crate::models::ProjectTemplate;
use anyhow::Result;
use console::Style;
use ignore::WalkBuilder;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut allowed_paths = HashSet::new();
    let walker = WalkBuilder::new(&current_dir)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "info" && name != "target" && name != "node_modules"
        })
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.path() != current_dir {
                allowed_paths.insert(entry.path().to_path_buf());
            }
        }
    }

    let mut tree_output = String::new();
    let mut content_output = String::new();
    let mut template_files = BTreeMap::new();
    let mut total_chars = 0;

    tree_output.push_str(&format!("{}/\n", project_name));
    render_tree(&current_dir, &allowed_paths, "", &mut tree_output)?;

    let mut sorted_paths: Vec<_> = allowed_paths.iter().collect();
    sorted_paths.sort();

    for path in sorted_paths {
        if path.is_file() {
            let relative_path = path
                .strip_prefix(&current_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Ok(content) = fs::read_to_string(path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                content_output.push_str(&format!("\n### {}\n", relative_path));
                content_output.push_str(&format!("```{}\n", ext));
                content_output.push_str(&content);
                content_output.push_str("\n```\n");
                
                total_chars += content.len();

                template_files.insert(relative_path.clone(), content);
            }
        } else if path.is_dir() {
             let relative_path = path
                .strip_prefix(&current_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
             let dir_key = if relative_path.ends_with('/') { relative_path } else { format!("{}/", relative_path) };
             template_files.insert(dir_key, "".to_string());
        }
    }

    let info_dir = current_dir.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }

    let final_markdown = format!(
        "# Project Context: {}\n\n## File Structure\n\n```text\n{}\n```\n\n## File Contents\n{}",
        project_name, tree_output, content_output
    );
    let md_path = info_dir.join("context.md");
    fs::write(&md_path, &final_markdown)?;

    let template = ProjectTemplate {
        template_name: project_name.clone(),
        description: format!("Context snapshot of {}", project_name),
        files: template_files,
    };
    let json_content = serde_json::to_string_pretty(&template)?;
    let json_path = info_dir.join(format!("{}_template.json", project_name));
    fs::write(&json_path, &json_content)?;

    update_gitignore(&current_dir)?;

    let green = Style::new().green();
    println!("Context generated at:");
    println!("  - {}", green.apply_to(format!("{:?}", md_path)));
    println!("  - {}", green.apply_to(format!("{:?}", json_path)));
    
    let estimated_tokens = total_chars / 4;
    println!("Estimated Tokens: ~{}", estimated_tokens);

    Ok(())
}

fn render_tree(
    current_dir: &Path,
    allowed: &HashSet<PathBuf>,
    prefix: &str,
    output: &mut String,
) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(current_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| allowed.contains(&e.path()))
        .collect();

    entries.sort_by(|a, b| {
        let a_name = a.file_name().to_string_lossy().to_string();
        let b_name = b.file_name().to_string_lossy().to_string();
        a_name.cmp(&b_name)
    });

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();

        let connector = if is_last { "└── " } else { "├── " };
        let display_name = if is_dir { format!("{}/", name) } else { name };

        output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));

        if is_dir {
            let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            render_tree(&path, allowed, &child_prefix, output)?;
        }
    }

    Ok(())
}

fn update_gitignore(root: &Path) -> Result<()> {
    let gitignore_path = root.join(".gitignore");
    let entry = "info/";
    
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.contains(entry) {
            use std::io::Write;
            let mut file = fs::OpenOptions::new().append(true).open(&gitignore_path)?;
            writeln!(file, "\n{}", entry)?;
            println!("Added 'info/' to .gitignore");
        }
    } else {
        fs::write(&gitignore_path, format!("{}\n", entry))?;
    }
    Ok(())
}
```

### src/cmd/install.rs
```rs
use crate::models::ProjectTemplate;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(path_str: &str) -> Result<()> {
    let source_path = Path::new(path_str);
    if !source_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path_str));
    }

    let content = fs::read_to_string(source_path).context("Failed to read template file")?;
    let template: ProjectTemplate =
        serde_json::from_str(&content).context("Invalid template JSON format")?;

    if template.template_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Template name cannot be empty"));
    }

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let templates_dir = PathBuf::from(home).join(".wally/templates");

    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)?;
    }

    let target_path = templates_dir.join(format!("{}.json", template.template_name));

    if target_path.exists() {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Template '{}' already exists. Overwrite?",
                template.template_name
            ))
            .default(false)
            .interact()?;

        if !confirm {
            println!("Installation aborted.");
            return Ok(());
        }
    }

    fs::write(&target_path, content)?;
    println!(
        "Template '{}' installed successfully to {:?}",
        template.template_name, target_path
    );

    Ok(())
}
```

### src/cmd/list.rs
```rs
use crate::cmd::DEFAULT_TEMPLATE;
use crate::models::ProjectTemplate;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    println!("{:<20} | {}", "TEMPLATE", "DESCRIPTION");
    println!("{:-<20}-+-{:-<40}", "", "");

    let default_tmpl: ProjectTemplate = serde_json::from_str(DEFAULT_TEMPLATE)?;
    print_row(&default_tmpl.template_name, &default_tmpl.description);

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let templates_dir = PathBuf::from(home).join(".wally/templates");

    if templates_dir.exists() {
        for entry in fs::read_dir(templates_dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(tmpl) = serde_json::from_str::<ProjectTemplate>(&content) {
                            print_row(&tmpl.template_name, &tmpl.description);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_row(name: &str, desc: &str) {
    let clean_desc = desc.replace('\n', " ");
    let display_desc = if clean_desc.len() > 70 {
        format!("{}...", &clean_desc[..67])
    } else {
        clean_desc
    };
    println!("{:<20} | {}", name, display_desc);
}
```

### src/cmd/menu.rs
```rs
use crate::cli::Commands;
use crate::cmd;
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};

struct MenuItem {
    label: &'static str,
    desc: &'static str,
    command: Commands,
}

pub fn run() -> Result<()> {
    let term = Term::stdout();
    let desc_style = Style::new().dim();

    loop {
        term.clear_screen()?;
        
        let items = get_menu_items();
        let options: Vec<String> = items
            .iter()
            .map(|i| format!("{:<10} {}", i.label, desc_style.apply_to(format!("- {}", i.desc))))
            .collect();
        
        let mut selection_items = options.clone();
        selection_items.push(Style::new().red().apply_to("Exit").to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select command")
            .default(0)
            .items(&selection_items)
            .clear(true)
            .interact()?;

        if selection == items.len() {
            break;
        }

        let selected_item = &items[selection];
        
        match &selected_item.command {
            Commands::New { project_name, template } => cmd::new::run(project_name.clone(), template.clone())?,
            Commands::Context => cmd::context::run()?,
            Commands::List => cmd::list::run()?,
            Commands::Add => cmd::add::run()?,
            Commands::Commit => cmd::commit::run()?,
            Commands::Push => cmd::push::run()?,
            Commands::Branch => cmd::branch::run()?,
            Commands::Update => cmd::update::run()?,
            Commands::Reset => cmd::reset::run()?,
            Commands::Stats => cmd::stats::run()?,
            Commands::Tag => cmd::tag::run()?,
            Commands::Install { path } => cmd::install::run(path)?,
            Commands::Uninstall { template_name } => cmd::uninstall::run(template_name)?,
        }

        println!("\nPress ENTER to continue...");
        let _ = term.read_line()?;
    }

    Ok(())
}

fn get_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: "New",
            desc: "Initialize a new project",
            command: Commands::New { project_name: None, template: None },
        },
        MenuItem {
            label: "Add",
            desc: "Stage files (View Diffs)",
            command: Commands::Add,
        },
        MenuItem {
            label: "Commit",
            desc: "Commit changes",
            command: Commands::Commit,
        },
        MenuItem {
            label: "Push",
            desc: "Push to remote (Handles Squash/Force)",
            command: Commands::Push,
        },
        MenuItem {
            label: "Update",
            desc: "Safe pull (Stash -> Rebase -> Pop)",
            command: Commands::Update,
        },
        MenuItem {
            label: "Branch",
            desc: "Switch, Create, Merge, Squash, Delete",
            command: Commands::Branch,
        },
        MenuItem {
            label: "Reset",
            desc: "Undo changes (Reflog/Hard Reset)",
            command: Commands::Reset,
        },
        MenuItem {
            label: "Stats",
            desc: "Project activity statistics",
            command: Commands::Stats,
        },
        MenuItem {
            label: "Context",
            desc: "Generate AI context (MD & JSON)",
            command: Commands::Context,
        },
        MenuItem {
            label: "Tag",
            desc: "Semantic versioning tags",
            command: Commands::Tag,
        },
        MenuItem {
            label: "List",
            desc: "List installed templates",
            command: Commands::List,
        },
    ]
}
```

### src/cmd/mod.rs
```rs
pub mod add;
pub mod branch;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod menu;
pub mod new;
pub mod push;
pub mod reset;
pub mod stats;
pub mod tag;
pub mod uninstall;
pub mod update;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");

use anyhow::{Context, Result};
use console::Style;
use std::process::{Command, ExitStatus, Output};

pub fn execute_git(args: &[&str]) -> Result<ExitStatus> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .status()
        .context("Failed to execute git command")
}

pub fn execute_git_output(args: &[&str]) -> Result<Output> {
    print_git_cmd(args);
    Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")
}

pub fn print_git_cmd(args: &[&str]) {
    let cmd_style = Style::new().blue().bold();
    let symbol = Style::new().cyan().bold();
    println!("{} {}", symbol.apply_to(">"), cmd_style.apply_to(format!("git {}", args.join(" "))));
}
```

### src/cmd/new.rs
```rs
use crate::cmd::DEFAULT_TEMPLATE;
use crate::models::ProjectTemplate;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::cmd::print_git_cmd; // Import helper for printing

pub fn run(project_name: Option<String>, template_name: Option<String>) -> Result<()> {
    let name = match project_name {
        Some(n) => n,
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Project name")
            .interact_text()
            .context("Failed to read project name")?,
    };

    let template_content = get_template_content(template_name)?;
    let template: ProjectTemplate = serde_json::from_str(&template_content)?;

    let root_path = Path::new(&name);
    if root_path.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists.", name));
    }

    fs::create_dir_all(root_path)?;

    // Custom execution for init to change directory context properly
    let init_args = &["init", "-b", "main"];
    print_git_cmd(init_args);
    Command::new("git")
        .args(init_args)
        .current_dir(root_path)
        .output()
        .context("Failed to init git with branch main")?;

    for (path_str, content) in template.files {
        let full_path = root_path.join(&path_str);

        if path_str.ends_with('/') || (content.is_empty() && !path_str.contains('.')) {
            fs::create_dir_all(full_path)?;
        } else {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut f = File::create(&full_path)?;
            f.write_all(content.as_bytes())?;

            #[cfg(unix)]
            if path_str.ends_with(".sh") {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&full_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&full_path, perms)?;
            }
        }
    }

    println!("Project '{}' created successfully with 'main' branch.", name);
    Ok(())
}

fn get_template_content(template_name: Option<String>) -> Result<String> {
    if let Some(name) = template_name {
        load_template(&name)
    } else {
        let mut templates = vec!["default".to_string()];
        
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
        let templates_dir = PathBuf::from(&home).join(".wally/templates");
        
        if templates_dir.exists() {
            for entry in fs::read_dir(templates_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            templates.push(stem.to_string());
                        }
                    }
                }
            }
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a template")
            .default(0)
            .items(&templates)
            .interact()?;

        if templates[selection] == "default" {
            Ok(DEFAULT_TEMPLATE.to_string())
        } else {
            load_template(&templates[selection])
        }
    }
}

fn load_template(name: &str) -> Result<String> {
    if name == "default" {
        return Ok(DEFAULT_TEMPLATE.to_string());
    }
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let path = PathBuf::from(home)
        .join(".wally/templates")
        .join(format!("{}.json", name));
    
    fs::read_to_string(path).context("Template not found")
}
```

### src/cmd/push.rs
```rs
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
```

### src/cmd/reset.rs
```rs
use crate::cmd::{execute_git, execute_git_output};
use anyhow::{Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};

pub fn run() -> Result<()> {
    let modes = vec![
        "Undo Recent Actions (Reflog) - Find 'lost' commits",
        "Reset to Specific Commit (Log) - Go back in history",
    ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select reset mode")
        .default(0)
        .items(&modes)
        .clear(true)
        .interact()
        .context("Failed to read mode selection")?;

    if mode_selection == 0 {
        handle_reflog_reset()
    } else {
        handle_log_reset()
    }
}

fn handle_reflog_reset() -> Result<()> {
    let output = execute_git_output(&["reflog", "-n", "20", "--pretty=format:%h - %gs: %s"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No recent actions found in reflog.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action to UNDO")
        .default(0)
        .items(&entries)
        .clear(true)
        .interact()?;

    ask_reset_type_and_execute(entries[selection])
}

fn handle_log_reset() -> Result<()> {
    let output = execute_git_output(&["log", "--pretty=format:%h - %s (%cr)", "-n", "20"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No commits found to reset to.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit to reset to")
        .default(0)
        .items(&entries)
        .clear(true)
        .interact()?;

    ask_reset_type_and_execute(entries[selection])
}

fn ask_reset_type_and_execute(entry: &str) -> Result<()> {
    let hash = entry
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid entry format"))?;

    let options = vec![
        "Soft: Keep my changes (Safe. Changes stay in 'Staging Area', ready to commit)",
        "Mixed: Keep my files, but unstage (Safe. Changes stay in files, but not 'Added')",
        "Hard: Discard all changes (DANGEROUS! Files will be exactly like the target commit)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("How should we reset to {}?", hash))
        .default(0)
        .items(&options)
        .interact()?;

    let (mode_arg, is_dangerous) = match selection {
        0 => ("--soft", false),
        1 => ("--mixed", false),
        2 => ("--hard", true),
        _ => unreachable!(),
    };

    if is_dangerous {
        let red = Style::new().red().bold();
        println!("{}", red.apply_to("WARNING: Hard reset will PERMANENTLY DELETE all uncommitted changes."));
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you absolutely sure you want to proceed?")
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    println!("Performing {} reset to {}...", mode_arg, hash);
    let status = execute_git(&["reset", mode_arg, hash])?;

    if status.success() {
        println!("{}", Style::new().green().apply_to(format!("Successfully reset to {}", hash)));
    } else {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    Ok(())
}
```

### src/cmd/stats.rs
```rs
use crate::cmd::execute_git_output;
use anyhow::Result;
use console::Style;
use std::collections::HashMap;

pub fn run() -> Result<()> {
    let yellow = Style::new().yellow();
    let green = Style::new().green();
    let red = Style::new().red();
    let header = Style::new().cyan().bold();

    let commit_count = get_commit_count_days(7)?;
    println!(
        "\n{} Commits in the last 7 days: {}",
        yellow.apply_to("●"),
        commit_count
    );

    let (added, deleted) = get_line_stats_days(7)?;
    println!(
        "{} Lines changed: {} (added), {} (deleted)",
        yellow.apply_to("●"),
        green.apply_to(format!("+{}", added)),
        red.apply_to(format!("-{}", deleted))
    );

    println!("\n{}", header.apply_to("Top 5 Most Modified Files (Last 30 Days):"));
    let top_files = get_top_modified_files(30, 5)?;
    if top_files.is_empty() {
        println!("  No data available.");
    } else {
        for (file, count) in top_files {
            println!("  {:>3} times - {}", count, file);
        }
    }

    let (total_commits, first_commit) = get_total_stats()?;
    println!("\n{}", header.apply_to("Lifetime Summary:"));
    println!("  Total Commits:  {}", total_commits);
    println!("  Project Start:  {}", first_commit);

    Ok(())
}

fn get_commit_count_days(days: u32) -> Result<usize> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--oneline"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().count())
}

fn get_line_stats_days(days: u32) -> Result<(u64, u64)> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--numstat", "--pretty=format:"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut added = 0;
    let mut deleted = 0;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(a) = parts[0].parse::<u64>() {
                added += a;
            }
            if let Ok(d) = parts[1].parse::<u64>() {
                deleted += d;
            }
        }
    }
    Ok((added, deleted))
}

fn get_top_modified_files(days: u32, limit: usize) -> Result<Vec<(String, usize)>> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--pretty=format:", "--name-only"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut counts = HashMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(sorted.into_iter().take(limit).collect())
}

fn get_total_stats() -> Result<(usize, String)> {
    let count_out = execute_git_output(&["rev-list", "--count", "HEAD"])?;
    let count = String::from_utf8(count_out.stdout)?
        .trim()
        .parse()
        .unwrap_or(0);

    let date_out = execute_git_output(&["log", "--reverse", "--format=%ad", "--date=short"])?;
    let first_date = String::from_utf8(date_out.stdout)?
        .lines()
        .next()
        .unwrap_or("Unknown")
        .to_string();

    Ok((count, first_date))
}
```

### src/cmd/tag.rs
```rs
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
```

### src/cmd/uninstall.rs
```rs
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn run(template_name: &str) -> Result<()> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let templates_dir = PathBuf::from(home).join(".wally/templates");
    let target_path = templates_dir.join(format!("{}.json", template_name));

    if !target_path.exists() {
        return Err(anyhow::anyhow!(
            "Template '{}' not found in {:?}",
            template_name,
            templates_dir
        ));
    }

    fs::remove_file(target_path)?;
    println!("Template '{}' uninstalled successfully.", template_name);

    Ok(())
}
```

### src/cmd/update.rs
```rs
use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select, Confirm};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run() -> Result<()> {
    let success = Style::new().green().bold();
    let warning = Style::new().yellow();
    let dim = Style::new().dim();

    // 1. 分支检查与重命名建议
    println!("{} Checking branch...", dim.apply_to("[1/4]"));
    let current_branch = get_current_branch()?;
    if current_branch == "master" {
        println!("{}", warning.apply_to("   Current branch is 'master'."));
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("   Rename to 'main' to follow modern standards?")
            .default(true)
            .interact()? 
        {
            execute_git(&["branch", "-m", "master", "main"])?;
        }
    }

    // 2. 检查暂存区
    println!("{} Checking workspace...", dim.apply_to("[2/4]"));
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("   {}", warning.apply_to("Uncommitted changes found. Stashing safely..."));
        execute_git(&["stash", "push", "-m", "wally-auto-update"])?;
        stashed = true;
    }

    // 3. 拉取并变基
    println!("{} Pulling from remote...", dim.apply_to("[3/4]"));
    let pull_status = execute_git(&["pull", "--rebase"])?;

    if !pull_status.success() {
        if is_rebase_in_progress()? {
            handle_rebase_conflict_loop()?;
        } else {
            let remote = "origin";
            let branch = get_current_branch()?;
            println!("   Standard pull failed. Trying {}/{}...", remote, branch);
            let retry_status = execute_git(&["pull", "--rebase", remote, &branch])?;
            if !retry_status.success() && is_rebase_in_progress()? {
                handle_rebase_conflict_loop()?;
            }
        }
    }

    // 4. 恢复暂存区
    println!("{} Finalizing...", dim.apply_to("[4/4]"));
    if stashed {
        println!("   Restoring your stashed changes...");
        let status = execute_git(&["stash", "pop"])?;
        if !status.success() {
            println!("{}", Style::new().red().bold().apply_to("   STASH CONFLICT!"));
            println!("   Your changes collided with remote changes. Please fix markers manually.");
        }
    }

    println!("\n{}", success.apply_to("Update successful!"));
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = execute_git_output(&["branch", "--show-current"])?;
    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(if branch.is_empty() { "main".to_string() } else { branch })
}

fn check_if_dirty() -> Result<bool> {
    let output = execute_git_output(&["status", "--porcelain"])?;
    Ok(!output.stdout.is_empty())
}

fn is_rebase_in_progress() -> Result<bool> {
    let output = execute_git_output(&["rev-parse", "--git-dir"])?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    let path = Path::new(&git_dir);
    Ok(path.join("rebase-merge").exists() || path.join("rebase-apply").exists())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = execute_git_output(&["status", "--porcelain"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines()
        .filter(|l| l.starts_with("UU") || l.starts_with("AA") || l.starts_with("DU") || l.starts_with("UD"))
        .filter_map(|l| if l.len() > 3 { Some(l[3..].to_string()) } else { None })
        .collect())
}

fn handle_rebase_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red().bold();
    
    loop {
        let files = get_conflicted_files()?;
        if files.is_empty() { break; }

        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING UPDATE"));
        
        for file in &files {
            print_conflict_details(file)?;
        }

        let choices = vec!["I have resolved the conflicts", "Abort update"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Action required")
            .items(&choices)
            .default(0)
            .interact()?;

        if selection == 1 {
            execute_git(&["rebase", "--abort"])?;
            return Err(anyhow::anyhow!("Update aborted by user."));
        }

        execute_git(&["add", "."])?;
        // 使用环境变量避免弹出编辑器，直接尝试继续
        let status = std::process::Command::new("git")
            .env("GIT_EDITOR", "true") 
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("Conflicts resolved and update continued.");
            break;
        } else {
            println!("{}", red.apply_to("Still have conflicts. Please check the markers in files."));
            thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(())
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan().bold();
    println!("\n{}", cyan.apply_to(format!("File: {}", file_path)));
    
    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut inside_conflict = false;
        
        for line_res in reader.lines().take(50) { // 最多展示50行以防刷屏
            let line = line_res.unwrap_or_default();
            if line.starts_with("<<<<<<<") { inside_conflict = true; }
            if inside_conflict {
                println!("    {}", line);
            }
            if line.starts_with(">>>>>>>") { inside_conflict = false; }
        }
    }
    Ok(())
}
```

### src/main.rs
```rs
mod cli;
mod cmd;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::New { project_name, template }) => {
            cmd::new::run(project_name, template)?;
        }
        Some(Commands::Context) => {
            cmd::context::run()?;
        }
        Some(Commands::List) => {
            cmd::list::run()?;
        }
        Some(Commands::Add) => {
            cmd::add::run()?;
        }
        Some(Commands::Commit) => {
            cmd::commit::run()?;
        }
        Some(Commands::Push) => {
            cmd::push::run()?;
        }
        Some(Commands::Branch) => {
            cmd::branch::run()?;
        }
        Some(Commands::Update) => {
            cmd::update::run()?;
        }
        Some(Commands::Reset) => {
            cmd::reset::run()?;
        }
        Some(Commands::Stats) => {
            cmd::stats::run()?;
        }
        Some(Commands::Tag) => {
            cmd::tag::run()?;
        }
        Some(Commands::Install { path }) => {
            cmd::install::run(&path)?;
        }
        Some(Commands::Uninstall { template_name }) => {
            cmd::uninstall::run(&template_name)?;
        }
        None => {
            cmd::menu::run()?;
        }
    }

    Ok(())
}
```

### src/models.rs
```rs
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectTemplate {
    pub template_name: String,
    #[serde(default)]
    pub description: String,
    pub files: BTreeMap<String, String>,
}
```

### templates/default.json
```json
{
  "template_name": "default",
  "description": "A basic project structure with git and temp folders.",
  "files": {
    ".gitignore": "temp/\ntarget/\n**/*.DS_Store\ninfo/\n",
    "src/": "",
    "temp/": "",
    "info/": "",
    "script/": "",
    "README.md": "# Project\n\nThis is a basic project structure."
  }
}

```
