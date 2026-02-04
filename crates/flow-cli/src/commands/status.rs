use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;

pub fn run(mobile: bool) -> Result<()> {
    let config = flow_core::Config::load()?;
    let state = flow_core::state::State::load(&config.state_dir)?;

    if mobile {
        print_mobile_status(&config, &state);
    } else {
        print_full_status(&config, &state);
    }

    Ok(())
}

fn print_mobile_status(config: &flow_core::Config, state: &flow_core::state::State) {
    // Compact header
    println!(
        "{} {}",
        icons::GAUGE.with(colors::CYAN),
        "flow".bold().with(colors::WHITE)
    );

    // Current project
    if let Some(ref current) = state.current_project {
        println!(
            "{} {}",
            icons::FOLDER.with(colors::GREEN),
            current.as_str().with(colors::WHITE)
        );
    }

    // Git info from current directory
    if let Ok(worktrees) = flow_git::worktree::list() {
        if !worktrees.is_empty() {
            println!(
                "{} {} worktrees",
                icons::WORKTREE.with(colors::MAGENTA),
                worktrees.len().to_string().with(colors::YELLOW)
            );
        }
    }

    // Tmux sessions
    if let Ok(sessions) = flow_tmux::session::list_sessions() {
        if !sessions.is_empty() {
            println!(
                "{} {} sessions",
                icons::TMUX.with(colors::BLUE),
                sessions.len().to_string().with(colors::YELLOW)
            );
        }
    }

    // Projects count
    if let Ok(projects) = flow_core::project::discover_all() {
        println!(
            "{} {} projects in {}",
            icons::FOLDER.with(colors::DIM),
            projects.len().to_string().with(colors::DIM),
            ui::truncate_path(&config.projects_dir, 20).with(colors::DIM)
        );
    }
}

fn print_full_status(config: &flow_core::Config, state: &flow_core::state::State) {
    // Beautiful header
    ui::print_header("  Flow Status Dashboard  ");

    // ═══ Configuration Section ═══
    ui::print_section(icons::FOLDER, "Configuration");
    ui::print_kv("Projects", &ui::truncate_path(&config.projects_dir, 40));
    ui::print_kv("State", &ui::truncate_path(&config.state_dir, 40));
    ui::print_kv("Default branch", &config.default_base_branch);

    // ═══ Current Context ═══
    ui::print_section(icons::ARROW, "Current Context");
    if let Some(ref current) = state.current_project {
        ui::print_item(icons::CHECK, &format!("Project: {current}"), colors::GREEN);
    } else {
        ui::print_empty("No active project");
    }

    // Show current git branch if in a git repo
    if let Ok(branch) = flow_git::branch::current() {
        if !branch.is_empty() {
            ui::print_item(
                icons::GIT_BRANCH,
                &format!("Branch: {branch}"),
                colors::PURPLE,
            );
        }
    }

    // ═══ Git Worktrees ═══
    ui::print_section(icons::WORKTREE, "Git Worktrees");
    match flow_git::worktree::list() {
        Ok(worktrees) if !worktrees.is_empty() => {
            for wt in &worktrees {
                let branch_display = format!("({})", wt.branch).with(colors::PURPLE);
                let path_display = ui::truncate_path(&wt.path, 35).with(colors::DIM);
                println!(
                    "  {} {} {} {}",
                    icons::GIT_BRANCH.with(colors::CYAN),
                    wt.name.as_str().bold().with(colors::WHITE),
                    branch_display,
                    path_display
                );
            }
        }
        Ok(_) => ui::print_empty("No worktrees (this is the main checkout)"),
        Err(_) => ui::print_empty("Not in a git repository"),
    }

    // ═══ TMUX Sessions ═══
    ui::print_section(icons::TMUX, "TMUX Sessions");
    match flow_tmux::session::list_sessions() {
        Ok(sessions) if !sessions.is_empty() => {
            // Highlight current session
            let current_session = std::env::var("TMUX")
                .ok()
                .and_then(|t| t.split(',').next().map(String::from));

            for session in &sessions {
                let is_current = current_session
                    .as_ref()
                    .is_some_and(|_| session.contains("attached") || sessions.len() == 1);

                if is_current {
                    println!(
                        "  {} {} {}",
                        icons::DOT.with(colors::GREEN),
                        session.as_str().bold().with(colors::GREEN),
                        "(current)".with(colors::DIM)
                    );
                } else {
                    ui::print_item(icons::DOT, session, colors::BLUE);
                }
            }
        }
        Ok(_) => ui::print_empty("No active sessions"),
        Err(_) => ui::print_empty("TMUX not running"),
    }

    // ═══ Projects ═══
    ui::print_section(icons::FOLDER, "Discovered Projects");
    match flow_core::project::discover_all() {
        Ok(projects) if !projects.is_empty() => {
            let display_count = projects.len().min(8);
            for project in projects.iter().take(display_count) {
                ui::print_item(icons::DOT, &project.name, colors::CYAN);
            }
            if projects.len() > display_count {
                println!(
                    "  {} {}",
                    "...".with(colors::DIM),
                    format!("and {} more", projects.len() - display_count).with(colors::DIM)
                );
            }
        }
        Ok(_) => ui::print_empty("No projects found in scan directories"),
        Err(e) => ui::print_empty(&format!("Error scanning: {e}")),
    }

    // ═══ Recent Projects ═══
    if !state.recent_projects.is_empty() {
        ui::print_section(icons::CLOCK, "Recent Projects");
        for (i, proj) in state.recent_projects.iter().take(5).enumerate() {
            let num = format!("{}.", i + 1).with(colors::DIM);
            println!("  {} {}", num, proj.as_str().with(colors::WHITE));
        }
    }
}
