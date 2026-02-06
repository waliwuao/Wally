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
            format!("Project Setup ({})", setup_list),
            format!("Development & Sync ({})", dev_list),
            format!("Template Management ({})", tmpl_list),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&categories)
            .default(0)
            .interact()?;

        match selection {
            0 => break,
            1 => show_category_menu("Project Setup", get_setup_cmds())?,
            2 => show_category_menu("Development & Sync", get_dev_cmds())?,
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
            desc: "Manage branches interactively: switch, create with prefixes (feat/fix), or delete safely.",
        },
        CmdInfo {
            name: "reset",
            usage: "wally reset",
            desc: "Choose from the last 20 commits to perform a HARD reset. All uncommitted changes will be lost.",
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