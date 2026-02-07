use anyhow::Result;
use flow_core::AgentConfig;

pub fn run(port: u16, no_open: bool, public_dir: Option<String>) -> Result<()> {
    let config = AgentConfig::new()
        .with_port(port)
        .with_open_browser(!no_open);

    let config = if let Some(dir) = public_dir {
        config.with_public_dir(std::path::PathBuf::from(dir))
    } else {
        config
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(flow_server::run_server(config))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
