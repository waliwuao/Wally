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