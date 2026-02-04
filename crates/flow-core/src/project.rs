use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

/// Discover all projects in the configured projects directory.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the directory cannot be read.
pub fn discover_all() -> Result<Vec<Project>, crate::FlowError> {
    let config = crate::Config::load()?;
    let mut projects = Vec::new();

    if config.projects_dir.exists() {
        for entry in std::fs::read_dir(&config.projects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                projects.push(Project { name, path });
            }
        }
    }

    Ok(projects)
}
