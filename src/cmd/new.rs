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