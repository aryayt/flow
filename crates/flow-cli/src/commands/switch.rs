use anyhow::Result;
use skim::prelude::*;
use std::io::Cursor;
use tracing::info;

pub fn run(query: Option<&str>) -> Result<()> {
    info!("Switching project with query: {:?}", query);

    // Get list of projects
    let projects = flow_core::project::discover_all()?;

    if projects.is_empty() {
        println!("No projects found in the projects directory");
        return Ok(());
    }

    // If query matches exactly one project, switch directly
    if let Some(q) = query {
        let matches: Vec<_> = projects.iter().filter(|p| p.name.contains(q)).collect();
        if matches.len() == 1 {
            flow_tmux::session::switch_to(&matches[0].path)?;
            println!("Switched to {}", matches[0].name);
            return Ok(());
        }
    }

    // Use skim for fuzzy finding
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .query(query)
        .build()
        .expect("Failed to build skim options");

    let project_list: String = projects
        .iter()
        .map(|p| format!("{}\t{}", p.name, p.path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(project_list));

    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    if let Some(item) = selected.first() {
        let output = item.output();
        let parts: Vec<&str> = output.split('\t').collect();
        if parts.len() >= 2 {
            let path = std::path::PathBuf::from(parts[1]);
            flow_tmux::session::switch_to(&path)?;
            println!("Switched to {}", parts[0]);
        }
    }

    Ok(())
}
