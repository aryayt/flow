use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.tui_theme;

    // Split main area into logs and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Logs
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let logs_area = chunks[0];
    let footer_area = chunks[1];

    let block = Block::default()
        .title(format!("Logs ({} messages)", app.log_messages.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(logs_area);
    frame.render_widget(block, logs_area);

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

    // Render footer with key hints
    let footer_text = if let Some(status) = app.get_status_message() {
        status.to_string()
    } else {
        " 1-4:views  r:refresh  t:theme  ?:help  q:quit".to_string()
    };

    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.muted));
    frame.render_widget(footer, footer_area);
}
