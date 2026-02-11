use crate::cmd::{execute_git, execute_git_output};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Select};

struct CommitType<'a> {
    code: &'a str,
    desc: &'a str,
}

const COMMIT_TYPES: &[CommitType] = &[
    CommitType { code: "feat", desc: "A new feature" },
    CommitType { code: "fix", desc: "A bug fix" },
    CommitType { code: "docs", desc: "Documentation only changes" },
    CommitType { code: "style", desc: "Changes that do not affect the meaning of the code" },
    CommitType { code: "refactor", desc: "Code change that neither fixes a bug nor adds feature" },
    CommitType { code: "perf", desc: "A code change that improves performance" },
    CommitType { code: "test", desc: "Adding missing tests or correcting existing tests" },
    CommitType { code: "build", desc: "Changes that affect the build system" },
    CommitType { code: "ci", desc: "Changes to our CI configuration files" },
    CommitType { code: "chore", desc: "Other changes that don't modify src or test files" },
    CommitType { code: "revert", desc: "Reverts a previous commit" },
];

pub fn run() -> Result<()> {
    // 1. 检查是否有暂存的文件
    let output = execute_git_output(&["diff", "--cached", "--name-only"])?;
    let staged = String::from_utf8(output.stdout)?;

    if staged.trim().is_empty() {
        println!("Nothing staged to commit.");
        println!("Please run 'wally add' first to select files.");
        return Ok(());
    }

    // 2. 构建并执行 commit
    let message = build_commit_message()?;
    let status = execute_git(&["commit", "-m", &message])?;

    if !status.success() {
        return Err(anyhow::anyhow!("git commit failed"));
    }

    println!("Commit successful!");

    Ok(())
}

fn build_commit_message() -> Result<String> {
    let items: Vec<String> = COMMIT_TYPES
        .iter()
        .map(|t| format!("{:<10} {}", t.code, t.desc))
        .collect();

    // 选择类型
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select change type")
        .default(0)
        .items(&items)
        .clear(true)
        .interact()?;

    let selected_type = COMMIT_TYPES[selection].code;

    // 输入范围
    let scope: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Scope (optional)")
        .allow_empty(true)
        .interact()?;

    // 输入简述
    let subject: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() { Err("Subject cannot be empty") } else { Ok(()) }
        })
        .interact()?;

    // 输入详情
    let body: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Body (optional)")
        .allow_empty(true)
        .interact()?;

    // 拼接消息
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