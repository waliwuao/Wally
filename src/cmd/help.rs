use anyhow::Result;
use console::Style;

pub fn run() -> Result<()> {
    let category = Style::new().cyan().bold();
    let dim = Style::new().dim();

    println!("\nWally - A Git Toolset for Beginners\n");

    println!("{}", category.apply_to("[ Project Setup ]"));
    print_cmd("new", "Initialize a new project with a template");
    print_cmd("context", "Generate project structure and code context for AI");

    println!("\n{}", category.apply_to("[ Daily Workflow ]"));
    print_cmd("update", "Sync with remote (Auto stash, rebase, and pop)");
    print_cmd("commit", "Interactive stage, conventional commit, and push");
    print_cmd("branch", "Interactive branch management (Switch, Create, Delete)");
    print_cmd("reset", "Hard reset to a previous commit from list");

    println!("\n{}", category.apply_to("[ Template Management ]"));
    print_cmd("list", "List all installed project templates");
    print_cmd("install", "Install a new template from a JSON file");
    print_cmd("uninstall", "Remove an installed template");

    println!("\n{}", category.apply_to("[ Others ]"));
    print_cmd("help", "Show this help message");

    println!("\n{}", dim.apply_to("Usage: wally <COMMAND>"));
    Ok(())
}

fn print_cmd(name: &str, desc: &str) {
    let cmd_style = Style::new().yellow();
    println!("  {:<12} {}", cmd_style.apply_to(name), desc);
}