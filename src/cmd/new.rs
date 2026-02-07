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