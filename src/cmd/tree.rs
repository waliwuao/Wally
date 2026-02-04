use anyhow::Result;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    generate_tree_logic(&current_dir)
}

pub fn generate_tree_logic(root_path: &Path) -> Result<()> {
    let info_dir = root_path.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }

    let mut output = String::new();
    output.push_str(&format!("Project: {:?}\n", root_path.file_name().unwrap_or_default()));

    let walker = WalkBuilder::new(root_path)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| e.file_name() != ".git")
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();
            if path == root_path { continue; }
            
            let relative = path.strip_prefix(root_path).unwrap_or(path);
            let depth = relative.components().count();
            let indent = "  ".repeat(depth - 1);
            let name = relative.file_name().unwrap_or_default().to_string_lossy();
            
            output.push_str(&format!("{}|-- {}\n", indent, name));
        }
    }

    fs::write(info_dir.join("tree.txt"), output)?;
    Ok(())
}