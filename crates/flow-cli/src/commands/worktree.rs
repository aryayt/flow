use crate::ui::{self, colors, icons};
use anyhow::Result;
use clap::Subcommand;
use crossterm::style::Stylize;

#[derive(Subcommand)]
pub enum WorktreeAction {
    /// List all worktrees
    List,
    /// Remove a worktree
    Remove {
        /// Name of the worktree to remove
        name: String,
    },
}

pub fn run(action: WorktreeAction) -> Result<()> {
    match action {
        WorktreeAction::List => list_worktrees(),
        WorktreeAction::Remove { name } => remove_worktree(&name),
    }
}

fn list_worktrees() -> Result<()> {
    let worktrees = flow_git::worktree::list()?;

    if worktrees.is_empty() {
        ui::print_info("No worktrees found (this is the main checkout)");
        return Ok(());
    }

    // Header
    println!(
        "{} {} {}",
        icons::WORKTREE.with(colors::CYAN),
        "Git Worktrees".bold().with(colors::WHITE),
        format!("({})", worktrees.len()).with(colors::DIM)
    );
    println!();

    // Find the longest name for alignment
    let max_name_len = worktrees.iter().map(|w| w.name.len()).max().unwrap_or(10);

    for wt in &worktrees {
        // Determine if this is the current worktree
        let current_dir = std::env::current_dir().ok();
        let is_current = current_dir
            .as_ref()
            .is_some_and(|cwd| cwd.starts_with(&wt.path) || wt.path.starts_with(cwd));

        let status_icon = if is_current {
            icons::CHECK.with(colors::GREEN)
        } else {
            icons::DOT.with(colors::DIM)
        };

        let name_padded = format!("{:width$}", wt.name, width = max_name_len);
        let name_styled = if is_current {
            name_padded.bold().with(colors::GREEN)
        } else {
            name_padded.with(colors::WHITE)
        };

        let branch_styled = format!("({})", wt.branch).with(colors::PURPLE);
        let path_styled = ui::truncate_path(&wt.path, 40).with(colors::DIM);

        println!("  {status_icon} {name_styled} {branch_styled} {path_styled}");
    }

    Ok(())
}

fn remove_worktree(name: &str) -> Result<()> {
    // Show what we're about to do
    ui::print_info(&format!("Removing worktree: {}", name.with(colors::YELLOW)));

    // Check if worktree exists first
    let worktrees = flow_git::worktree::list()?;
    let found = worktrees.iter().any(|w| w.name == name);

    if !found {
        ui::print_error(&format!("Worktree '{name}' not found"));
        println!();
        println!("{}", "Available worktrees:".with(colors::DIM));
        for wt in &worktrees {
            println!(
                "  {} {}",
                icons::DOT.with(colors::DIM),
                wt.name.as_str().with(colors::CYAN)
            );
        }
        return Ok(());
    }

    // Remove it
    flow_git::worktree::remove(name)?;

    ui::print_success(&format!("Removed worktree: {}", name.with(colors::GREEN)));

    Ok(())
}
