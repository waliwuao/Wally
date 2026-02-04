use crate::models::ProjectTemplate;
use anyhow::Result;
use ignore::WalkBuilder;
use std::fs::{self, File};
use std::path::Path;

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let name = current_dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut template = ProjectTemplate {
        template_name: name.clone(),
        files: Default::default(),
    };

    let walker = WalkBuilder::new(&current_dir)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| e.file_name() != ".git")
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();
            if path == current_dir { continue; }

            let relative = path.strip_prefix(&current_dir)?.to_string_lossy().replace("\\", "/");
            
            if path.is_dir() {
                let key = if relative.ends_with('/') { relative } else { format!("{}/", relative) };
                template.files.insert(key, String::new());
            } else {
                if let Ok(content) = fs::read_to_string(path) {
                    template.files.insert(relative, content);
                }
            }
        }
    }

    let info_dir = current_dir.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }
    
    let file = File::create(info_dir.join(format!("{}.json", name)))?;
    serde_json::to_writer_pretty(file, &template)?;
    
    Ok(())
}