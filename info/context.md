# Project Context: wally

## File Structure

```text
wally/
├── .gitignore
├── Cargo.toml
├── src/
│   ├── cli.rs
│   ├── cmd/
│   │   ├── add.rs
│   │   ├── commit.rs
│   │   ├── context.rs
│   │   ├── install.rs
│   │   ├── list.rs
│   │   ├── mod.rs
│   │   ├── new.rs
│   │   ├── push.rs
│   │   ├── reset.rs
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
        project_name: String,
        #[arg(short, long)]
        template: Option<String>,
    },
    Context,
    List,
    Add {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        files: Vec<String>,
    },
    Commit,
    Push {
        url: Option<String>,
    },
    Reset,
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
```

### src/cmd/commit.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
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
    let items: Vec<String> = COMMIT_TYPES
        .iter()
        .map(|t| format!("{:<10} {}", t.code, t.desc))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the type of change that you're committing")
        .default(0)
        .items(&items)
        .interact()
        .context("Failed to read selection")?;

    let selected_type = COMMIT_TYPES[selection].code;

    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to read scope")?;

    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject (short description)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Subject cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to read subject")?;

    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (detailed description, optional)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to read body")?;

    let mut message = if scope.trim().is_empty() {
        format!("{}: {}", selected_type, subject)
    } else {
        format!("{}({}): {}", selected_type, scope, subject)
    };

    if !body.trim().is_empty() {
        message.push_str("\n\n");
        message.push_str(&body);
    }

    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .status()
        .context("Failed to execute git commit")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git commit failed"));
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
pub mod add;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod new;
pub mod push;
pub mod reset;
pub mod uninstall;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");
```

### src/cmd/new.rs
```rs
use crate::cmd::DEFAULT_TEMPLATE;
use crate::models::ProjectTemplate;
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(project_name: &str, template_name: Option<String>) -> Result<()> {
    let root_path = Path::new(project_name);
    if root_path.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists.", project_name));
    }

    let template_content = if let Some(t_name) = template_name {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
        let t_path = PathBuf::from(home)
            .join(".wally/templates")
            .join(format!("{}.json", t_name));
        fs::read_to_string(t_path).context("Template not found")?
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    let template: ProjectTemplate = serde_json::from_str(&template_content)?;

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

    println!("Project '{}' created successfully.", project_name);

    Ok(())
}
```

### src/cmd/push.rs
```rs
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
            cmd::new::run(&project_name, template)?;
        }
        Commands::Context => {
            cmd::context::run()?;
        }
        Commands::List => {
            cmd::list::run()?;
        }
        Commands::Add { files } => {
            cmd::add::run(files)?;
        }
        Commands::Commit => {
            cmd::commit::run()?;
        }
        Commands::Push { url } => {
            cmd::push::run(url)?;
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
