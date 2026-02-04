mod commands;
mod ui;

use clap::{Parser, Subcommand};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "flow")]
#[command(about = "Workflow CLI for multi-agent development", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Disable timing output
    #[arg(long, global = true)]
    no_timing: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a git worktree and open in tmux
    Branch {
        /// Name of the branch/worktree to create
        name: String,
        /// Base branch to create from
        #[arg(short, long, default_value = "main")]
        base: String,
    },
    /// Fuzzy-find and switch to a project
    Switch {
        /// Optional query to pre-filter
        query: Option<String>,
    },
    /// Manage git worktrees
    Worktree {
        #[command(subcommand)]
        action: commands::worktree::WorktreeAction,
    },
    /// Sync state across machines
    Sync,
    /// Run security scans
    Scan {
        /// Run all scanners
        #[arg(long)]
        all: bool,
    },
    /// Show status dashboard
    Status {
        /// Mobile-friendly output
        #[arg(long)]
        mobile: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    let result = match cli.command {
        Commands::Branch { name, base } => commands::branch::run(&name, &base),
        Commands::Switch { query } => commands::switch::run(query.as_deref()),
        Commands::Worktree { action } => commands::worktree::run(action),
        Commands::Sync => commands::sync::run(),
        Commands::Scan { all } => commands::scan::run(all),
        Commands::Status { mobile } => commands::status::run(mobile),
    };

    if !cli.no_timing {
        ui::print_timing(start.elapsed());
    }

    result
}
