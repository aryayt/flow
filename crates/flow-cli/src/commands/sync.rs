use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;

pub fn run() -> Result<()> {
    println!(
        "{} {}",
        icons::SYNC.with(colors::CYAN),
        "Syncing state across machines..."
            .bold()
            .with(colors::WHITE)
    );

    flow_sync::sync_state()?;

    ui::print_success("State synced successfully");
    Ok(())
}
