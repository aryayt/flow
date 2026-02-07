use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;
use flow_core::Theme;

#[allow(clippy::unnecessary_wraps)]
pub fn list() -> Result<()> {
    ui::print_section(icons::DOT, "Available Themes");
    for theme in Theme::ALL {
        let colors = theme.colors();
        println!(
            "  {} {} {}",
            icons::DOT.with(colors::CYAN),
            theme.name().bold().with(colors::WHITE),
            format!("(css: {})", colors.css_primary).with(colors::DIM),
        );
    }
    Ok(())
}

pub fn set(name: &str) -> Result<()> {
    let theme: Theme = name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    ui::print_success(&format!("Theme set to: {}", theme.name()));
    Ok(())
}
