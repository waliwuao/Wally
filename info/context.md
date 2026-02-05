# Project Context: wally

## File Structure

```text
wally/
├── .gitignore
├── Cargo.toml
├── src/
│   ├── cli.rs
│   ├── cmd/
│   │   ├── branch.rs
│   │   ├── commit.rs
│   │   ├── context.rs
│   │   ├── install.rs
│   │   ├── list.rs
│   │   ├── mod.rs
│   │   ├── new.rs
│   │   ├── reset.rs
│   │   ├── sync.rs
│   │   └── uninstall.rs
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

### src/cli.rs
```rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wally")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    Commit,
    Branch,
    Sync,
    Reset,
    Install {
        path: String,
    },
    Uninstall {
        template_name: String,
    },
}
```

### src/cmd/branch.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let actions = vec!["Switch Branch", "Create New Branch", "Delete Branch"];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch action")
        .default(0)
        .items(&actions)
        .interact()
        .context("Failed to read selection")?;

    match selection {
        0 => switch_branch(),
        1 => create_branch(),
        2 => delete_branch(),
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
        .interact()
        .context("Failed to select branch")?;

    let target = &branches[selection];

    if target == &current {
        println!("Already on branch '{}'", target);
        return Ok(());
    }

    let status = Command::new("git")
        .args(&["checkout", target])
        .status()
        .context("Failed to switch branch")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to checkout branch {}", target));
    }

    Ok(())
}

fn create_branch() -> Result<()> {
    let types = vec!["feat", "fix", "chore", "docs", "refactor", "style", "test", "other"];
    
    let type_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branch type")
        .default(0)
        .items(&types)
        .interact()
        .context("Failed to select branch type")?;

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
        .interact_text()
        .context("Failed to read branch name")?;

    let full_name = if prefix == "other" {
        name
    } else {
        format!("{}/{}", prefix, name)
    };

    let status = Command::new("git")
        .args(&["checkout", "-b", &full_name])
        .status()
        .context("Failed to create branch")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create branch {}", full_name));
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
        .with_prompt("Select branches to delete (SPACE to select, ENTER to confirm)")
        .items(&available_to_delete)
        .interact()
        .context("Failed to select branches")?;

    if selections.is_empty() {
        println!("No branches selected.");
        return Ok(());
    }

    for index in selections {
        let branch_name = &available_to_delete[index];
        let status = Command::new("git")
            .args(&["branch", "-D", branch_name])
            .status()
            .context("Failed to execute git branch -D")?;

        if status.success() {
            println!("Deleted branch '{}'", branch_name);
        } else {
            eprintln!("Failed to delete branch '{}'", branch_name);
        }
    }

    Ok(())
}

fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["branch", "--format=%(refname:short)"])
        .output()
        .context("Failed to list branches")?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
```

### src/cmd/commit.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::process::Command;

struct CommitType<'a> {
    code: &'a str,
    desc: &'a str,
}

const COMMIT_TYPES: &[CommitType] = &[
    CommitType { code: "feat", desc: "A new feature" },
    CommitType { code: "fix", desc: "A bug fix" },
    CommitType { code: "docs", desc: "Documentation only changes" },
    CommitType { code: "style", desc: "Changes that do not affect the meaning of the code" },
    CommitType { code: "refactor", desc: "A code change that neither fixes a bug nor adds a feature" },
    CommitType { code: "perf", desc: "A code change that improves performance" },
    CommitType { code: "test", desc: "Adding missing tests or correcting existing tests" },
    CommitType { code: "build", desc: "Changes that affect the build system or external dependencies" },
    CommitType { code: "ci", desc: "Changes to our CI configuration files and scripts" },
    CommitType { code: "chore", desc: "Other changes that don't modify src or test files" },
    CommitType { code: "revert", desc: "Reverts a previous commit" },
];

pub fn run() -> Result<()> {
    if !stage_files()? {
        println!("No files staged. Skipping commit.");
        return Ok(());
    }

    let message = build_commit_message()?;
    
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .status()
        .context("Failed to execute git commit")?;

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

fn stage_files() -> Result<bool> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to get git status")?;

    let stdout = String::from_utf8(output.stdout)?;
    if stdout.is_empty() {
        return Ok(false);
    }

    let mut files = Vec::new();
    let mut items = vec!["[ALL] (Select this to stage ALL changes)".to_string()];

    for line in stdout.lines() {
        if line.len() > 3 {
            let status_code = &line[0..2];
            let file = &line[3..];
            
            let status_text = match status_code {
                "??" => "Untracked",
                " M" | "M " => "Modified ",
                " A" | "A " => "Added    ",
                " D" | "D " => "Deleted  ",
                " R" | "R " => "Renamed  ",
                "UU" => "Conflict ",
                _ => status_code.trim(),
            };

            files.push(file.to_string());
            items.push(format!("[{}] {}", status_text.trim(), file));
        }
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select files to stage (SPACE to select, ENTER to confirm)")
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        return Ok(false);
    }

    if selections.contains(&0) {
        Command::new("git").args(&["add", "."]).status()?;
    } else {
        for &index in &selections {
            let file_index = index - 1;
            Command::new("git").arg("add").arg(&files[file_index]).status()?;
        }
    }

    Ok(true)
}

fn build_commit_message() -> Result<String> {
    let items: Vec<String> = COMMIT_TYPES
        .iter()
        .map(|t| format!("{:<10} {}", t.code, t.desc))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the type of change")
        .default(0)
        .items(&items)
        .interact()?;

    let selected_type = COMMIT_TYPES[selection].code;

    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact()?;

    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject (short description)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() { Err("Subject cannot be empty") } else { Ok(()) }
        })
        .interact()?;

    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (detailed description, optional)")
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
    let remote_output = Command::new("git").args(&["remote"]).output()?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL to add origin")
            .interact_text()?;
        
        Command::new("git").args(&["remote", "add", "origin", &url]).status()?;
        Command::new("git").args(&["push", "-u", "origin", "main"]).status()?;
    } else {
        let status = Command::new("git").arg("push").status()?;
        if !status.success() {
            println!("Standard push failed. Trying to set upstream...");
            let branch_output = Command::new("git").args(&["branch", "--show-current"]).output()?;
            let branch = String::from_utf8(branch_output.stdout)?.trim().to_string();
            Command::new("git").args(&["push", "-u", "origin", &branch]).status()?;
        }
    }
    Ok(())
}
```

### src/cmd/context.rs
```rs
use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let mut allowed_paths = HashSet::new();
    let walker = WalkBuilder::new(&current_dir)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Filter out git, info folder itself, and target/build artifacts
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
    tree_output.push_str(&format!("{}/\n", project_name));
    render_tree(&current_dir, &allowed_paths, "", &mut tree_output)?;

    let mut content_output = String::new();
    let mut total_chars = 0;

    // Sort paths for consistent output
    let mut sorted_paths: Vec<_> = allowed_paths.iter().collect();
    sorted_paths.sort();

    for path in sorted_paths {
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                let relative_path = path
                    .strip_prefix(&current_dir)
                    .unwrap_or(path)
                    .to_string_lossy();
                
                // Determine language for markdown code block
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                
                content_output.push_str(&format!("\n### {}\n", relative_path));
                content_output.push_str(&format!("```{}\n", ext));
                content_output.push_str(&content);
                content_output.push_str("\n```\n");
                
                total_chars += content.len();
            }
        }
    }

    let final_markdown = format!(
        "# Project Context: {}\n\n## File Structure\n\n```text\n{}\n```\n\n## File Contents\n{}",
        project_name, tree_output, content_output
    );

    let info_dir = current_dir.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }
    
    let output_path = info_dir.join("context.md");
    fs::write(&output_path, &final_markdown)?;

    // Simple estimation: 1 token ~= 4 chars
    let estimated_tokens = total_chars / 4;

    println!("Context generated at {:?}", output_path);
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
    let display_desc = if clean_desc.len() > 50 {
        format!("{}...", &clean_desc[..47])
    } else {
        clean_desc
    };
    println!("{:<20} | {}", name, display_desc);
}
```

### src/cmd/mod.rs
```rs
pub mod branch;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod new;
pub mod reset;
pub mod sync;
pub mod uninstall;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");
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

    Command::new("git")
        .arg("init")
        .current_dir(root_path)
        .output()
        .context("Failed to init git")?;

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

    println!("Project '{}' created successfully.", name);
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

### src/cmd/reset.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let output = Command::new("git")
        .args(&["log", "--pretty=format:%h - %s", "-n", "20"])
        .output()
        .context("Failed to get git log")?;

    let stdout = String::from_utf8(output.stdout)?;
    let commits: Vec<&str> = stdout.lines().collect();

    if commits.is_empty() {
        println!("No commits found to reset to.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit to reset to (HARD reset)")
        .default(0)
        .items(&commits)
        .interact()
        .context("Failed to read selection")?;

    let selected_commit = commits[selection];
    let commit_hash = selected_commit
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid commit format"))?;

    println!("Resetting to {}...", commit_hash);

    let status = Command::new("git")
        .args(&["reset", "--hard", commit_hash])
        .status()
        .context("Failed to execute git reset")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    println!("Successfully reset to {}", commit_hash);

    Ok(())
}
```

### src/cmd/sync.rs
```rs
use anyhow::{Context, Result};
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("Local changes detected. Stashing...");
        stash_push()?;
        stashed = true;
    }

    println!("Fetching and rebasing...");
    if let Err(_) = pull_rebase() {
        handle_rebase_conflict_loop()?;
    }

    if stashed {
        println!("Restoring local changes...");
        if let Err(_) = stash_pop() {
            handle_stash_conflict_loop()?;
        }
    }

    println!("Sync completed successfully.");
    Ok(())
}

fn check_if_dirty() -> Result<bool> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    Ok(!output.stdout.is_empty())
}

fn stash_push() -> Result<()> {
    let status = Command::new("git")
        .args(&["stash", "push", "-m", "wally-auto-sync"])
        .status()
        .context("Failed to stash changes")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to execute git stash"));
    }
    Ok(())
}

fn stash_pop() -> Result<()> {
    let status = Command::new("git")
        .args(&["stash", "pop"])
        .status()
        .context("Failed to pop stash")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Stash pop failed"));
    }
    Ok(())
}

fn pull_rebase() -> Result<()> {
    let status = Command::new("git")
        .args(&["pull", "--rebase"])
        .status()
        .context("Failed to execute git pull --rebase")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Rebase failed"));
    }
    Ok(())
}

fn handle_rebase_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING REBASE"));
        println!("The following files have merge conflicts:\n");

        let conflicted_files = get_conflicted_files()?;
        if conflicted_files.is_empty() {
            println!("{}", green.apply_to("No conflicted files found."));
        } else {
            for file in &conflicted_files {
                print_conflict_details(file)?;
            }
        }

        println!("\n{}", yellow.apply_to("Please open the files above, resolve the conflicts, and save them."));

        let choices = vec!["I have resolved the conflicts (Continue)", "Abort Sync"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            Command::new("git").args(&["rebase", "--abort"]).status()?;
            return Err(anyhow::anyhow!("Sync aborted by user"));
        }

        println!("Staging changes...");
        Command::new("git").args(&["add", "."]).status()?;

        println!("Continuing rebase...");
        let status = Command::new("git")
            .env("GIT_EDITOR", "true") 
            .args(&["rebase", "--continue"])
            .status()?;

        if status.success() {
            println!("{}", green.apply_to("Rebase resolved successfully!"));
            break;
        } else {
            println!("{}", red.apply_to("Rebase continue failed. Conflicts might still exist."));
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    Ok(())
}

fn handle_stash_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED DURING STASH POP"));
        println!("Your local changes conflict with the incoming updates.\n");

        let conflicted_files = get_conflicted_files()?;
        for file in &conflicted_files {
            print_conflict_details(file)?;
        }

        println!("\n{}", yellow.apply_to("Please resolve the conflicts in the files above."));

        let choices = vec!["I have resolved the conflicts", "Abort (Changes remain in stash list)"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            return Err(anyhow::anyhow!("Stash pop cleanup aborted. You may need to reset or drop stash manually."));
        }

        let remaining_conflicts = get_conflicted_files()?;
        if remaining_conflicts.is_empty() {
            println!("{}", green.apply_to("Conflicts resolved."));
            Command::new("git").args(&["stash", "drop"]).status()?;
            break;
        } else {
            println!("{}", red.apply_to("Conflicts still detected. Please ensure markers are removed."));
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    Ok(())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    let mut files = Vec::new();

    for line in stdout.lines() {
        // Look for 'UU', 'AA', 'UD', etc.
        if line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DU") || line.starts_with("UD") {
            if line.len() > 3 {
                files.push(line[3..].to_string());
            }
        }
    }

    Ok(files)
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan();
    let blue = Style::new().blue();
    
    println!("{}", cyan.apply_to(format!("File: {}", file_path)));
    
    if !path.exists() {
        return Ok(());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    let mut inside_conflict = false;
    let mut printed_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("<<<<<<<") {
            inside_conflict = true;
            println!("  {}", blue.apply_to(format!("Line {}: Start of conflict", i + 1)));
        }

        if inside_conflict {
            println!("    {}", line);
        }

        if line.starts_with(">>>>>>>") {
            inside_conflict = false;
            println!("  {}", blue.apply_to(format!("Line {}: End of conflict", i + 1)));
            println!("");
            printed_count += 1;
            if printed_count >= 3 {
                println!("    ... (more conflicts hidden) ...");
                break;
            }
        }
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
        Commands::New { project_name, template } => {
            cmd::new::run(project_name, template)?;
        }
        Commands::Context => {
            cmd::context::run()?;
        }
        Commands::List => {
            cmd::list::run()?;
        }
        Commands::Commit => {
            cmd::commit::run()?;
        }
        Commands::Branch => {
            cmd::branch::run()?;
        }
        Commands::Sync => {
            cmd::sync::run()?;
        }
        Commands::Reset => {
            cmd::reset::run()?;
        }
        Commands::Install { path } => {
            cmd::install::run(&path)?;
        }
        Commands::Uninstall { template_name } => {
            cmd::uninstall::run(&template_name)?;
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
    "script/": ""
  }
}
```
