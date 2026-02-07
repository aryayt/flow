use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;
use flow_db::FeatureStore;

fn default_db_path() -> std::path::PathBuf {
    let config = flow_core::AgentConfig::new();
    config.features_db_path("default")
}

fn open_db(db_path: Option<&str>) -> Result<flow_db::Database> {
    let path = match db_path {
        Some(p) => std::path::PathBuf::from(p),
        None => default_db_path(),
    };

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    flow_db::Database::open(&path).map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn list(db_path: Option<&str>) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    let features = FeatureStore::get_all(&conn).map_err(|e| anyhow::anyhow!("{e}"))?;

    if features.is_empty() {
        ui::print_empty("No features found. Use 'flow features add' to create one.");
        return Ok(());
    }

    ui::print_header("  Features  ");

    for feature in &features {
        let status_icon = if feature.passes {
            icons::CHECK.with(colors::GREEN)
        } else if feature.in_progress {
            "◉".with(colors::CYAN)
        } else {
            icons::DOT.with(colors::YELLOW)
        };

        let deps = if feature.dependencies.is_empty() {
            String::new()
        } else {
            format!(
                " [deps: {}]",
                feature
                    .dependencies
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        println!(
            "  {} {} #{} {} {}{}",
            status_icon,
            format!("[p{}]", feature.priority).with(colors::DIM),
            feature.id.to_string().with(colors::CYAN),
            feature.name.as_str().bold().with(colors::WHITE),
            feature.category.as_str().with(colors::PURPLE),
            deps.with(colors::DIM),
        );
    }

    // Print summary
    let stats = FeatureStore::get_stats(&conn).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!(
        "\n  {} total, {} passing, {} in progress, {} blocked",
        stats.total.to_string().with(colors::WHITE),
        stats.passing.to_string().with(colors::GREEN),
        stats.in_progress.to_string().with(colors::CYAN),
        stats.blocked.to_string().with(colors::RED),
    );

    Ok(())
}

pub fn add(db_path: Option<&str>, name: &str, description: &str, category: &str) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();

    let input = flow_core::CreateFeatureInput {
        name: name.to_string(),
        description: description.to_string(),
        priority: None,
        category: category.to_string(),
        steps: vec![],
        dependencies: vec![],
    };

    let feature = FeatureStore::create(&conn, &input).map_err(|e| anyhow::anyhow!("{e}"))?;
    ui::print_success(&format!("Created feature #{}: {}", feature.id, feature.name));
    Ok(())
}

pub fn ready(db_path: Option<&str>) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    let features = FeatureStore::get_ready(&conn).map_err(|e| anyhow::anyhow!("{e}"))?;

    if features.is_empty() {
        ui::print_empty("No features ready to work on.");
        return Ok(());
    }

    ui::print_section(icons::ARROW, "Ready Features");
    for feature in &features {
        println!(
            "  {} #{} {}",
            icons::DOT.with(colors::GREEN),
            feature.id.to_string().with(colors::CYAN),
            feature.name.as_str().with(colors::WHITE),
        );
    }

    Ok(())
}

pub fn claim(db_path: Option<&str>, id: i64) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    let feature =
        FeatureStore::claim_and_get(&conn, id).map_err(|e| anyhow::anyhow!("{e}"))?;
    ui::print_success(&format!("Claimed feature #{}: {}", feature.id, feature.name));
    Ok(())
}

pub fn pass(db_path: Option<&str>, id: i64) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    FeatureStore::mark_passing(&conn, id).map_err(|e| anyhow::anyhow!("{e}"))?;
    ui::print_success(&format!("Feature #{id} marked as passing"));
    Ok(())
}

pub fn fail(db_path: Option<&str>, id: i64) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    FeatureStore::mark_failing(&conn, id).map_err(|e| anyhow::anyhow!("{e}"))?;
    ui::print_warning(&format!("Feature #{id} marked as failing"));
    Ok(())
}

pub fn graph(db_path: Option<&str>) -> Result<()> {
    let db = open_db(db_path)?;
    let conn = db.writer().lock().unwrap();
    let features = FeatureStore::get_all(&conn).map_err(|e| anyhow::anyhow!("{e}"))?;

    let sorted =
        flow_resolver::topological_sort(&features).map_err(|e| anyhow::anyhow!("{e}"))?;

    ui::print_section(icons::ARROW, "Dependency Graph (topological order)");
    for feature in &sorted {
        let deps = if feature.dependencies.is_empty() {
            String::new()
        } else {
            format!(
                " <- [{}]",
                feature
                    .dependencies
                    .iter()
                    .map(|d| format!("#{d}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let status = if feature.passes {
            icons::CHECK.with(colors::GREEN)
        } else if feature.in_progress {
            "◉".with(colors::CYAN)
        } else {
            icons::DOT.with(colors::YELLOW)
        };

        println!(
            "  {} #{} {}{}",
            status,
            feature.id.to_string().with(colors::CYAN),
            feature.name.as_str().with(colors::WHITE),
            deps.with(colors::DIM),
        );
    }

    Ok(())
}
