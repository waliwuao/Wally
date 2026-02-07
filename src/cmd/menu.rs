use crate::cli::Commands;
use crate::cmd;
use anyhow::Result;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};

struct MenuItem {
    label: &'static str,
    desc: &'static str,
    command: Commands,
}

pub fn run() -> Result<()> {
    let term = Term::stdout();
    let desc_style = Style::new().dim();

    loop {
        term.clear_screen()?;
        
        let items = get_menu_items();
        let options: Vec<String> = items
            .iter()
            .map(|i| format!("{:<10} {}", i.label, desc_style.apply_to(format!("- {}", i.desc))))
            .collect();
        
        let mut selection_items = options.clone();
        selection_items.push(Style::new().red().apply_to("Exit").to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select command")
            .default(0)
            .items(&selection_items)
            .clear(true)
            .interact()?;

        if selection == items.len() {
            break;
        }

        let selected_item = &items[selection];
        
        match &selected_item.command {
            Commands::New { project_name, template } => cmd::new::run(project_name.clone(), template.clone())?,
            Commands::Context => cmd::context::run()?,
            Commands::List => cmd::list::run()?,
            Commands::Add => cmd::add::run()?,
            Commands::Commit => cmd::commit::run()?,
            Commands::Branch => cmd::branch::run()?,
            Commands::Update => cmd::update::run()?,
            Commands::Reset => cmd::reset::run()?,
            Commands::Stats => cmd::stats::run()?,
            Commands::Tag => cmd::tag::run()?,
            Commands::Install { path } => cmd::install::run(path)?,
            Commands::Uninstall { template_name } => cmd::uninstall::run(template_name)?,
        }

        println!("\nPress ENTER to continue...");
        let _ = term.read_line()?;
    }

    Ok(())
}

fn get_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: "New",
            desc: "Initialize a new project",
            command: Commands::New { project_name: None, template: None },
        },
        MenuItem {
            label: "Add",
            desc: "Stage files (View Diffs)",
            command: Commands::Add,
        },
        MenuItem {
            label: "Commit",
            desc: "Commit and Push (Auto-fix squash)",
            command: Commands::Commit,
        },
        MenuItem {
            label: "Update",
            desc: "Safe pull (Stash -> Rebase -> Pop)",
            command: Commands::Update,
        },
        MenuItem {
            label: "Branch",
            desc: "Switch, Create, Merge, Squash, Delete",
            command: Commands::Branch,
        },
        MenuItem {
            label: "Reset",
            desc: "Undo changes (Reflog/Hard Reset)",
            command: Commands::Reset,
        },
        MenuItem {
            label: "Stats",
            desc: "Project activity statistics",
            command: Commands::Stats,
        },
        MenuItem {
            label: "Context",
            desc: "Generate AI context (MD & JSON)",
            command: Commands::Context,
        },
        MenuItem {
            label: "Tag",
            desc: "Semantic versioning tags",
            command: Commands::Tag,
        },
        MenuItem {
            label: "List",
            desc: "List installed templates",
            command: Commands::List,
        },
    ]
}