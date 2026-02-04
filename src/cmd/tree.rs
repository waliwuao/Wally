use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    generate_tree_logic(&current_dir)
}

pub fn generate_tree_logic(root_path: &Path) -> Result<()> {
    let mut allowed_paths = HashSet::new();
    let walker = WalkBuilder::new(root_path)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "info" && name != "target"
        })
        .build();

    for result in walker {
        if let Ok(entry) = result {
            allowed_paths.insert(entry.path().to_path_buf());
        }
    }

    let mut output = String::new();
    let root_name = root_path.file_name().unwrap_or_default().to_string_lossy();
    output.push_str(&format!("{}/\n", root_name));

    render_directory(root_path, &allowed_paths, "", &mut output)?;

    let info_dir = root_path.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }
    fs::write(info_dir.join("tree.txt"), output)?;

    Ok(())
}

fn render_directory(
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
            render_directory(&path, allowed, &child_prefix, output)?;
        }
    }

    Ok(())
}