use crate::cmd::execute_git_output;
use anyhow::{Context, Result};
use console::Style;
use std::collections::HashMap;

pub fn run() -> Result<()> {
    let yellow = Style::new().yellow();
    let green = Style::new().green();
    let red = Style::new().red();
    let header = Style::new().cyan().bold();

    let commit_count = get_commit_count_days(7)?;
    println!(
        "\n{} Commits in the last 7 days: {}",
        yellow.apply_to("●"),
        commit_count
    );

    let (added, deleted) = get_line_stats_days(7)?;
    println!(
        "{} Lines changed: {} (added), {} (deleted)",
        yellow.apply_to("●"),
        green.apply_to(format!("+{}", added)),
        red.apply_to(format!("-{}", deleted))
    );

    println!("\n{}", header.apply_to("Top 5 Most Modified Files (Last 30 Days):"));
    let top_files = get_top_modified_files(30, 5)?;
    if top_files.is_empty() {
        println!("  No data available.");
    } else {
        for (file, count) in top_files {
            println!("  {:>3} times - {}", count, file);
        }
    }

    let (total_commits, first_commit) = get_total_stats()?;
    println!("\n{}", header.apply_to("Lifetime Summary:"));
    println!("  Total Commits:  {}", total_commits);
    println!("  Project Start:  {}", first_commit);

    Ok(())
}

fn get_commit_count_days(days: u32) -> Result<usize> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--oneline"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().count())
}

fn get_line_stats_days(days: u32) -> Result<(u64, u64)> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--numstat", "--pretty=format:"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut added = 0;
    let mut deleted = 0;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(a) = parts[0].parse::<u64>() {
                added += a;
            }
            if let Ok(d) = parts[1].parse::<u64>() {
                deleted += d;
            }
        }
    }
    Ok((added, deleted))
}

fn get_top_modified_files(days: u32, limit: usize) -> Result<Vec<(String, usize)>> {
    let since = format!("{} days ago", days);
    let output = execute_git_output(&["log", "--since", &since, "--pretty=format:", "--name-only"])?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut counts = HashMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(sorted.into_iter().take(limit).collect())
}

fn get_total_stats() -> Result<(usize, String)> {
    let count_out = execute_git_output(&["rev-list", "--count", "HEAD"])?;
    let count = String::from_utf8(count_out.stdout)?
        .trim()
        .parse()
        .unwrap_or(0);

    let date_out = execute_git_output(&["log", "--reverse", "--format=%ad", "--date=short"])?;
    let first_date = String::from_utf8(date_out.stdout)?
        .lines()
        .next()
        .unwrap_or("Unknown")
        .to_string();

    Ok((count, first_date))
}