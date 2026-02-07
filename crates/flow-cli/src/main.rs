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
    /// Start the web server for the task Kanban board
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3456")]
        port: u16,
        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
        /// Path to public directory for static assets
        #[arg(long)]
        public_dir: Option<String>,
    },
    /// Manage features in the SQLite database
    Features {
        #[command(subcommand)]
        action: FeatureAction,
        /// Path to the database file
        #[arg(long, global = true)]
        db: Option<String>,
    },
    /// Launch the TUI monitor
    Monitor,
    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
}

#[derive(Subcommand)]
enum FeatureAction {
    /// List all features
    List,
    /// Add a new feature
    Add {
        /// Feature name
        name: String,
        /// Feature description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Feature category
        #[arg(short, long, default_value = "")]
        category: String,
    },
    /// Show features ready to work on
    Ready,
    /// Claim a feature for work
    Claim {
        /// Feature ID
        id: i64,
    },
    /// Mark a feature as passing
    Pass {
        /// Feature ID
        id: i64,
    },
    /// Mark a feature as failing
    Fail {
        /// Feature ID
        id: i64,
    },
    /// Show dependency graph
    Graph,
}

#[derive(Subcommand)]
enum ThemeAction {
    /// List available themes
    List,
    /// Set the active theme
    Set {
        /// Theme name
        name: String,
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
        Commands::Serve {
            port,
            no_open,
            public_dir,
        } => commands::serve::run(port, no_open, public_dir),
        Commands::Features { action, db } => {
            let db_ref = db.as_deref();
            match action {
                FeatureAction::List => commands::features::list(db_ref),
                FeatureAction::Add {
                    name,
                    description,
                    category,
                } => commands::features::add(db_ref, &name, &description, &category),
                FeatureAction::Ready => commands::features::ready(db_ref),
                FeatureAction::Claim { id } => commands::features::claim(db_ref, id),
                FeatureAction::Pass { id } => commands::features::pass(db_ref, id),
                FeatureAction::Fail { id } => commands::features::fail(db_ref, id),
                FeatureAction::Graph => commands::features::graph(db_ref),
            }
        }
        Commands::Monitor => commands::monitor::run(),
        Commands::Theme { action } => match action {
            ThemeAction::List => commands::theme_cmd::list(),
            ThemeAction::Set { name } => commands::theme_cmd::set(&name),
        },
    };

    if !cli.no_timing {
        ui::print_timing(start.elapsed());
    }

    result
}
