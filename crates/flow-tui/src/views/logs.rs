use crate::app::App;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.tui_theme;

    let block = Block::default()
        .title(format!("Logs ({} messages)", app.log_messages.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.log_messages.is_empty() {
        return;
    }

    // Create list items from log messages
    let items: Vec<ListItem> = app
        .log_messages
        .iter()
        .enumerate()
        .map(|(idx, msg)| {
            // Color-code by message content (simple heuristic)
            let color = if msg.to_lowercase().contains("error") {
                theme.error
            } else if msg.to_lowercase().contains("warn") {
                theme.warning
            } else if msg.to_lowercase().contains("debug") {
                theme.muted
            } else {
                theme.foreground
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", idx + 1), Style::default().fg(theme.muted)),
                Span::styled(msg, Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}
