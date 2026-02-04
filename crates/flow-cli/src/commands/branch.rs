use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;

pub fn run(name: &str, base: &str) -> Result<()> {
    // Show what we're doing
    println!(
        "{} Creating branch {} from {}",
        icons::GIT_BRANCH.with(colors::CYAN),
        name.bold().with(colors::GREEN),
        base.with(colors::PURPLE)
    );

    // Create worktree
    let worktree_path = flow_git::worktree::create(name, base)?;

    ui::print_success(&format!(
        "Created worktree at {}",
        ui::truncate_path(&worktree_path, 50).with(colors::CYAN)
    ));

    // Open in tmux
    match flow_tmux::session::open_worktree(&worktree_path) {
        Ok(()) => {
            ui::print_success(&format!(
                "Opened TMUX session: {}",
                name.with(colors::MAGENTA)
            ));
        }
        Err(e) => {
            ui::print_warning(&format!("Could not open TMUX session: {e}"));
            println!(
                "  {} cd {}",
                icons::ARROW.with(colors::DIM),
                worktree_path.display().to_string().with(colors::CYAN)
            );
        }
    }

    Ok(())
}
