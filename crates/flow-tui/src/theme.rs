use flow_core::{AnsiColor, ThemeColors};
use ratatui::style::Color;

pub const fn to_ratatui_color(ansi: &AnsiColor) -> Color {
    Color::Indexed(ansi.0)
}

#[derive(Debug, Clone)]
pub struct TuiTheme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub muted: Color,
    pub border: Color,
    pub pending: Color,
    pub in_progress: Color,
    pub done: Color,
    pub blocked: Color,
    pub error: Color,
    pub warning: Color,
}

impl From<&ThemeColors> for TuiTheme {
    fn from(colors: &ThemeColors) -> Self {
        Self {
            primary: to_ratatui_color(&colors.primary),
            secondary: to_ratatui_color(&colors.secondary),
            background: to_ratatui_color(&colors.background),
            foreground: to_ratatui_color(&colors.foreground),
            accent: to_ratatui_color(&colors.accent),
            muted: to_ratatui_color(&colors.muted),
            border: to_ratatui_color(&colors.border),
            pending: to_ratatui_color(&colors.pending),
            in_progress: to_ratatui_color(&colors.in_progress),
            done: to_ratatui_color(&colors.done),
            blocked: to_ratatui_color(&colors.blocked),
            error: to_ratatui_color(&colors.error),
            warning: to_ratatui_color(&colors.warning),
        }
    }
}
