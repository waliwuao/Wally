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
│   │   ├── branch.rs
│   │   ├── commit.rs
│   │   ├── context.rs
│   │   ├── help.rs
│   │   ├── install.rs
│   │   ├── list.rs
│   │   ├── mod.rs
│   │   ├── new.rs
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
    Commit,
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
    Help,
}
```

### src/cmd/branch.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::process::Command;

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
    let types = vec![
        "feat", "fix", "chore", "docs", "refactor", "style", "test", "other",
    ];

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

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Push '{}' to remote (origin)?", full_name))
        .default(true)
        .interact()?
    {
        Command::new("git")
            .args(&["push", "-u", "origin", &full_name])
            .status()?;
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
        .interact()
        .context("Failed to select branch to merge")?;

    let branch_to_merge = &available_to_merge[selection];

    println!("Merging '{}' into '{}'...", branch_to_merge, current);

    let status = Command::new("git")
        .args(&["merge", branch_to_merge])
        .status()
        .context("Failed to execute git merge")?;

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
        .interact()?;

    let base = &base_branches[selection];

    if current == *base {
        return Err(anyhow::anyhow!("Cannot squash on the base branch itself. Switch to a feature branch."));
    }

    let merge_base_out = Command::new("git")
        .args(&["merge-base", base, &current])
        .output()?;
    
    let merge_base = String::from_utf8(merge_base_out.stdout)?.trim().to_string();

    if merge_base.is_empty() {
        return Err(anyhow::anyhow!("Could not find a common ancestor with {}", base));
    }

    println!("Squashing all commits from {}... All changes will be staged.", merge_base);

    let status = Command::new("git")
        .args(&["reset", "--soft", &merge_base])
        .status()?;

    if status.success() {
        println!("\nSuccess! All changes from your feature branch are now staged as a single block.");
        println!("Use 'wally commit' now to create a clean, single commit message.");
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

    let mut deleted_successfully = Vec::new();

    for &index in &selections {
        let branch_name = &available_to_delete[index];
        let status = Command::new("git")
            .args(&["branch", "-D", branch_name])
            .status()
            .context("Failed to execute git branch -D")?;

        if status.success() {
            println!("Deleted branch '{}' locally.", branch_name);
            deleted_successfully.push(branch_name.clone());
        } else {
            eprintln!("Failed to delete branch '{}' locally.", branch_name);
        }
    }

    if !deleted_successfully.is_empty() {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Also delete these branches from remote (origin)?")
            .default(false)
            .interact()?
        {
            for branch_name in deleted_successfully {
                let status = Command::new("git")
                    .args(&["push", "origin", "--delete", &branch_name])
                    .status()
                    .context("Failed to delete remote branch")?;
                
                if status.success() {
                    println!("Deleted branch '{}' from remote.", branch_name);
                }
            }
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
    let branch_output = Command::new("git").args(&["branch", "--show-current"]).output()?;
    let mut current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    if current_branch == "master" {
        println!("Detected 'master' branch. Renaming to 'main' for compatibility...");
        Command::new("git").args(&["branch", "-m", "master", "main"]).status()?;
        current_branch = "main".to_string();
    }

    let remote_output = Command::new("git").args(&["remote"]).output()?;
    let has_remote = !remote_output.stdout.is_empty();

    if !has_remote {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("No remote found. Enter remote URL to add origin")
            .interact_text()?;
        
        Command::new("git").args(&["remote", "add", "origin", &url]).status()?;
        Command::new("git").args(&["push", "-u", "origin", &current_branch]).status()?;
    } else {
        let status = Command::new("git").arg("push").status()?;
        if !status.success() {
            println!("Standard push failed. Trying to set upstream...");
            Command::new("git").args(&["push", "-u", "origin", &current_branch]).status()?;
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

### src/cmd/help.rs
```rs
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};

struct CmdInfo {
    name: &'static str,
    usage: &'static str,
    desc: &'static str,
}

pub fn run() -> Result<()> {
    let term = Term::stdout();
    let header_style = Style::new().cyan().bold();
    let exit_style = Style::new().red();

    loop {
        term.clear_screen()?;
        println!("{}", header_style.apply_to("--- Wally Interactive Help ---"));
        println!("Select a category to explore commands:\n");

        let setup_list = get_setup_cmds().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
        let dev_list = get_dev_cmds().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
        let tmpl_list = get_tmpl_cmds().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");

        let categories = vec![
            exit_style.apply_to("Exit Help").to_string(),
            format!("Project Setup         ({})", setup_list),
            format!("Git Operations        ({})", dev_list),
            format!("Template Management   ({})", tmpl_list),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&categories)
            .default(0)
            .interact()?;

        match selection {
            0 => break,
            1 => show_category_menu("Project Setup", get_setup_cmds())?,
            2 => show_category_menu("Git Operations", get_dev_cmds())?,
            3 => show_category_menu("Template Management", get_tmpl_cmds())?,
            _ => break,
        }
    }

    Ok(())
}

fn show_category_menu(cat_name: &str, cmds: Vec<CmdInfo>) -> Result<()> {
    let term = Term::stdout();
    let cat_style = Style::new().cyan().bold();
    let back_style = Style::new().blue();

    loop {
        term.clear_screen()?;
        println!("{}", cat_style.apply_to(format!("--- {} ---", cat_name)));
        
        let mut items: Vec<String> = vec![back_style.apply_to("Back to Main Menu").to_string()];
        items.extend(cmds.iter().map(|c| c.name.to_string()));

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a command for details")
            .items(&items)
            .default(0)
            .interact()?;

        if selection == 0 {
            break;
        }

        show_cmd_detail(&cmds[selection - 1])?;
    }
    Ok(())
}

fn show_cmd_detail(cmd: &CmdInfo) -> Result<()> {
    let term = Term::stdout();
    let cmd_style = Style::new().yellow().bold();
    let label_style = Style::new().dim();
    let back_style = Style::new().blue();

    term.clear_screen()?;
    println!("{}", cmd_style.apply_to(format!("Command: {}", cmd.name)));
    println!("\n{} {}", label_style.apply_to("Description:"), cmd.desc);
    println!("{} {}", label_style.apply_to("Usage:      "), cmd.usage);

    println!("\n");
    let _ = Select::with_theme(&ColorfulTheme::default())
        .items(&[back_style.apply_to("Back").to_string()])
        .default(0)
        .interact()?;

    Ok(())
}

fn get_setup_cmds() -> Vec<CmdInfo> {
    vec![
        CmdInfo {
            name: "new",
            usage: "wally new [project_name] [--template name]",
            desc: "Initialize a new project. If parameters are missing, it starts an interactive wizard.",
        },
        CmdInfo {
            name: "context",
            usage: "wally context",
            desc: "Scans the project and creates 'info/context.md' containing the file tree and code for AI analysis.",
        },
    ]
}

fn get_dev_cmds() -> Vec<CmdInfo> {
    vec![
        CmdInfo {
            name: "update",
            usage: "wally update",
            desc: "Sync with remote safely: stashes work, pulls with rebase, and restores work automatically.",
        },
        CmdInfo {
            name: "commit",
            usage: "wally commit",
            desc: "Unified helper: choose files to stage, write standardized commit messages, and push to remote.",
        },
        CmdInfo {
            name: "branch",
            usage: "wally branch",
            desc: "Manage branches interactively: switch, create, merge, squash, or delete safely.",
        },
        CmdInfo {
            name: "reset",
            usage: "wally reset",
            desc: "Provides options to undo recent actions (reflog) or reset to a specific commit (log).",
        },
        CmdInfo {
            name: "stats",
            usage: "wally stats",
            desc: "Shows project activity: recent commit frequency, line changes, and most modified files.",
        },
        CmdInfo {
            name: "tag",
            usage: "wally tag",
            desc: "Automated semantic versioning: calculates and pushes the next tag (Major/Minor/Patch).",
        },
    ]
}

fn get_tmpl_cmds() -> Vec<CmdInfo> {
    vec![
        CmdInfo {
            name: "list",
            usage: "wally list",
            desc: "Shows all project templates currently available in your system.",
        },
        CmdInfo {
            name: "install",
            usage: "wally install <file.json>",
            desc: "Adds a new custom project template from a local JSON configuration file.",
        },
        CmdInfo {
            name: "uninstall",
            usage: "wally uninstall <template_name>",
            desc: "Removes a previously installed custom template from the system.",
        },
    ]
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

### src/cmd/mod.rs
```rs
pub mod branch;
pub mod commit;
pub mod context;
pub mod help;
pub mod install;
pub mod list;
pub mod new;
pub mod reset;
pub mod stats;
pub mod tag;
pub mod update;
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
        .args(&["init", "-b", "main"])
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

### src/cmd/reset.rs
```rs
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

pub fn run() -> Result<()> {
    let modes = vec![
        "Undo Recent Actions (Reflog)",
        "Reset to Specific Commit (Log)",
    ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select reset mode")
        .default(0)
        .items(&modes)
        .interact()
        .context("Failed to read mode selection")?;

    if mode_selection == 0 {
        handle_reflog_reset()
    } else {
        handle_log_reset()
    }
}

fn handle_reflog_reset() -> Result<()> {
    let output = Command::new("git")
        .args(&["reflog", "-n", "20", "--pretty=format:%h - %gs: %s"])
        .output()
        .context("Failed to get git reflog")?;

    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No recent actions found in reflog.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action to UNDO (Reset to state before/at this action)")
        .default(0)
        .items(&entries)
        .interact()
        .context("Failed to read selection")?;

    perform_reset(entries[selection])
}

fn handle_log_reset() -> Result<()> {
    let output = Command::new("git")
        .args(&["log", "--pretty=format:%h - %s (%cr)", "-n", "20"])
        .output()
        .context("Failed to get git log")?;

    let stdout = String::from_utf8(output.stdout)?;
    let entries: Vec<&str> = stdout.lines().collect();

    if entries.is_empty() {
        println!("No commits found to reset to.");
        return Ok(());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit to reset to (HARD reset)")
        .default(0)
        .items(&entries)
        .interact()
        .context("Failed to read selection")?;

    perform_reset(entries[selection])
}

fn perform_reset(entry: &str) -> Result<()> {
    let hash = entry
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid entry format"))?;

    println!("Performing HARD reset to {}...", hash);

    let status = Command::new("git")
        .args(&["reset", "--hard", hash])
        .status()
        .context("Failed to execute git reset")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git reset failed"));
    }

    println!("Successfully reset to {}", hash);
    Ok(())
}
```

### src/cmd/stats.rs
```rs
use anyhow::{Context, Result};
use console::Style;
use std::collections::HashMap;
use std::process::Command;

pub fn run() -> Result<()> {
    let header = Style::new().cyan().bold();
    let yellow = Style::new().yellow();
    let green = Style::new().green();
    let red = Style::new().red();

    println!("{}", header.apply_to("\n--- Project Activity Statistics ---"));

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
    let output = Command::new("git")
        .args(&["log", "--since", &since, "--oneline"])
        .output()
        .context("Failed to get commit count")?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().count())
}

fn get_line_stats_days(days: u32) -> Result<(u64, u64)> {
    let since = format!("{} days ago", days);
    let output = Command::new("git")
        .args(&["log", "--since", &since, "--numstat", "--pretty=format:"])
        .output()
        .context("Failed to get line stats")?;

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
    let output = Command::new("git")
        .args(&["log", "--since", &since, "--pretty=format:", "--name-only"])
        .output()
        .context("Failed to get modified files")?;

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
    let count_out = Command::new("git")
        .args(&["rev-list", "--count", "HEAD"])
        .output()
        .context("Failed to get total commit count")?;
    let count = String::from_utf8(count_out.stdout)?
        .trim()
        .parse()
        .unwrap_or(0);

    let date_out = Command::new("git")
        .args(&["log", "--reverse", "--format=%ad", "--date=short"])
        .output()
        .context("Failed to get first commit date")?;
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
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn run() -> Result<()> {
    let header = Style::new().cyan().bold();
    let success = Style::new().green().bold();
    let warning = Style::new().yellow();
    let dim = Style::new().dim();

    println!("{}", header.apply_to("\nStarting Update Process..."));

    println!("{} Checking branch name...", dim.apply_to("[1/4]"));
    let current_branch = get_current_branch()?;
    if current_branch == "master" {
        println!("   {}", warning.apply_to("Renaming 'master' to 'main' for compatibility..."));
        Command::new("git").args(&["branch", "-m", "master", "main"]).status()?;
    }

    println!("{} Checking workspace status...", dim.apply_to("[2/4]"));
    let has_changes = check_if_dirty()?;
    let mut stashed = false;

    if has_changes {
        println!("   {}", warning.apply_to("Uncommitted changes found. Stashing locally..."));
        stash_push()?;
        stashed = true;
    }

    println!("{} Pulling latest changes from remote...", dim.apply_to("[3/4]"));
    let pull_status = Command::new("git").args(&["pull", "--rebase"]).status()?;

    if !pull_status.success() {
        if is_rebase_in_progress()? {
            handle_rebase_conflict_loop()?;
        } else {
            let remote = "origin";
            let branch = get_current_branch()?;
            println!("   {}", warning.apply_to(format!("Standard pull failed. Retrying with {}/{}...", remote, branch)));
            let retry_status = Command::new("git").args(&["pull", "--rebase", remote, &branch]).status()?;
            if !retry_status.success() {
                if is_rebase_in_progress()? {
                    handle_rebase_conflict_loop()?;
                } else {
                    return Err(anyhow::anyhow!("Update failed. Please check network or remote settings."));
                }
            }
        }
    }

    println!("{} Finalizing workspace...", dim.apply_to("[4/4]"));
    if stashed {
        println!("   {}", warning.apply_to("Restoring your stashed changes..."));
        if let Err(_) = stash_pop() {
            handle_stash_conflict_loop()?;
        }
    }

    println!("\n{}", success.apply_to("Update completed successfully!"));
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git").args(&["branch", "--show-current"]).output()?;
    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(if branch.is_empty() { "main".to_string() } else { branch })
}

fn check_if_dirty() -> Result<bool> {
    let output = Command::new("git").args(&["status", "--porcelain"]).output()?;
    Ok(!output.stdout.is_empty())
}

fn stash_push() -> Result<()> {
    Command::new("git").args(&["stash", "push", "-m", "wally-auto-update"]).status()?;
    Ok(())
}

fn stash_pop() -> Result<()> {
    let status = Command::new("git").args(&["stash", "pop"]).status()?;
    if !status.success() { return Err(anyhow::anyhow!("Stash conflict")); }
    Ok(())
}

fn is_rebase_in_progress() -> Result<bool> {
    let output = Command::new("git").args(&["rev-parse", "--git-dir"]).output()?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    let path = Path::new(&git_dir);
    Ok(path.join("rebase-merge").exists() || path.join("rebase-apply").exists())
}

fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git").args(&["status", "--porcelain"]).output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines()
        .filter(|l| l.starts_with("UU") || l.starts_with("AA") || l.starts_with("DU") || l.starts_with("UD"))
        .filter_map(|l| if l.len() > 3 { Some(l[3..].to_string()) } else { None })
        .collect())
}

fn handle_rebase_conflict_loop() -> Result<()> {
    let term = Term::stdout();
    let red = Style::new().red().bold();
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    loop {
        let files = get_conflicted_files()?;
        if files.is_empty() { break; }

        term.clear_screen()?;
        println!("{}", red.apply_to("CONFLICTS DETECTED"));
        println!("Please resolve conflicts in these files:\n");

        for file in &files {
            print_conflict_details(file)?;
        }

        println!("{}", yellow.apply_to("How to resolve:"));
        println!("1. Open files, look for markers, and keep the desired code.");
        println!("2. Save files and return here.\n");

        let choices = vec!["I have resolved all conflicts", "Abort Update"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action")
            .default(0)
            .items(&choices)
            .interact()?;

        if selection == 1 {
            Command::new("git").args(&["rebase", "--abort"]).status()?;
            return Err(anyhow::anyhow!("Update aborted."));
        }

        Command::new("git").args(&["add", "."]).status()?;
        let status = Command::new("git").env("GIT_EDITOR", "true").args(&["rebase", "--continue"]).status()?;

        if status.success() {
            println!("{}", green.apply_to("Rebase continued successfully!"));
            break;
        } else {
            println!("{}", red.apply_to("Conflicts still exist. Please check again."));
            thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(())
}

fn handle_stash_conflict_loop() -> Result<()> {
    let red = Style::new().red().bold();
    println!("\n{}", red.apply_to("STASH POP CONFLICT"));
    let files = get_conflicted_files()?;
    for file in &files { println!("  - {}", file); }
    println!("\nPlease resolve markers manually. Your work is safe in 'git stash list'.");
    Ok(())
}

fn print_conflict_details(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let cyan = Style::new().cyan().bold();
    let blue = Style::new().blue();
    println!("{}", cyan.apply_to(format!("File: {}", file_path)));
    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut inside = false;
        for (i, line_res) in reader.lines().enumerate() {
            let line = line_res.unwrap_or_else(|_| String::new());
            if line.starts_with("<<<<<<<") { 
                inside = true; 
                println!("  {}", blue.apply_to(format!("Line {}:", i + 1))); 
            }
            if inside { 
                println!("    {}", line); 
            }
            if line.starts_with(">>>>>>>") { 
                break; 
            }
        }
    }
    println!();
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
        Some(Commands::Commit) => {
            cmd::commit::run()?;
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
        Some(Commands::Help) => {
            cmd::help::run()?;
        }
        None => {
            cmd::help::run()?;
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
