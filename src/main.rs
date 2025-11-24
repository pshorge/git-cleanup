use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::process::Command;

/// Simple CLI tool to clean up merged git branches
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target branch (the one others were merged into), e.g., main or master
    #[arg(short, long, default_value = "main")]
    target: String,

    /// Dry-run mode (only prints what would be deleted, does not delete anything)
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!(
        "{} {} {}",
        "🔍 Searching for branches merged into".blue(),
        args.target.bold(),
        "..."
    );

    // 1. Git: Fetch list of merged branches
    let output = Command::new("git")
        .arg("branch")
        .arg("--merged")
        .arg(&args.target)
        .output()
        .context("Failed to execute git command. Are you in a git repository?")?;

    if !output.status.success() {
        eprintln!("{}", "Error: Target branch not found or not a git repository.".red());
        return Ok(());
    }

    let output_str = String::from_utf8(output.stdout)?;

    // 2. Parsing and filtering
    let branches_to_clean: Vec<String> = output_str
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| {
            // Ignore the current branch (marked with *)
            !line.starts_with('*')
        })
        .filter(|line| {
            // Ignore the target branch itself (main/master)
            line != &args.target
        })
        .collect();

    if branches_to_clean.is_empty() {
        println!("{}", "✨ Clean! No merged branches to delete.".green());
        return Ok(());
    }

    // 3. Interactive selection (UI)
    println!("Found {} branches to delete:", branches_to_clean.len());

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Space to select/unselect, Enter to confirm")
        .items(&branches_to_clean)
        // Select all by default
        .defaults(&vec![true; branches_to_clean.len()])
        .interact()?;

    if selections.is_empty() {
        println!("Cancelled. No branches were deleted.");
        return Ok(());
    }

    // 4. Deletion process
    for index in selections {
        let branch_name = &branches_to_clean[index];

        if args.dry_run {
            println!("{} {}", "[Dry-Run] Would delete:".yellow(), branch_name);
        } else {
            delete_branch(branch_name)?;
        }
    }

    if !args.dry_run {
        println!("{}", "Done! 🧹".green().bold());
    }

    Ok(())
}

fn delete_branch(branch_name: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("branch")
        .arg("-d") // -d is safe (checks merge status), -D forces deletion
        .arg(branch_name)
        .status()?;

    if status.success() {
        println!("{} {}", "🗑️  Deleted:".green(), branch_name);
    } else {
        eprintln!("{} {}", "❌ Error deleting:".red(), branch_name);
    }
    Ok(())
}