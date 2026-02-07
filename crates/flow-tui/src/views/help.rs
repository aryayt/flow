use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.tui_theme;

    // Create centered popup
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Help - Key Bindings")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.background));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Views:",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  1       ", Style::default().fg(theme.accent)),
            Span::styled("Kanban board view", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  2       ", Style::default().fg(theme.accent)),
            Span::styled("Agent dashboard", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  3       ", Style::default().fg(theme.accent)),
            Span::styled("Log viewer", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  4       ", Style::default().fg(theme.accent)),
            Span::styled("Dependency graph", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  Tab     ", Style::default().fg(theme.accent)),
            Span::styled("Cycle through views", Style::default().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  j/Down  ", Style::default().fg(theme.accent)),
            Span::styled("Next item", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  k/Up    ", Style::default().fg(theme.accent)),
            Span::styled("Previous item", Style::default().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other:",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  t       ", Style::default().fg(theme.accent)),
            Span::styled("Cycle theme", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  ?       ", Style::default().fg(theme.accent)),
            Span::styled("Toggle this help", Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  q/Ctrl-C", Style::default().fg(theme.accent)),
            Span::styled("Quit", Style::default().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
