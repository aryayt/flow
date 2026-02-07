use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.tui_theme;

    // Split main area into dashboard and footer
    let chunks_outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Dashboard
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let dashboard_area = chunks_outer[0];
    let footer_area = chunks_outer[1];

    let block = Block::default()
        .title("Agent Dashboard")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(dashboard_area);
    frame.render_widget(block, dashboard_area);

    // Use real stats from database if available, otherwise calculate from features
    let (total, passing, in_progress, _pending) = if let Some(ref stats) = app.feature_stats {
        (stats.total, stats.passing, stats.in_progress, stats.failing)
    } else {
        let total = app.features.len();
        let passing = app.features.iter().filter(|f| f.passes).count();
        let in_progress = app.features.iter().filter(|f| f.in_progress).count();
        let pending = app
            .features
            .iter()
            .filter(|f| !f.passes && !f.in_progress)
            .count();
        (total, passing, in_progress, pending)
    };

    #[allow(clippy::cast_precision_loss)]
    let progress_ratio = if total > 0 {
        passing as f64 / total as f64
    } else {
        0.0
    };

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Progress bar
            Constraint::Length(2), // Spacer
            Constraint::Length(3), // Agent counts
            Constraint::Length(2), // Spacer
            Constraint::Min(5),    // Recent activity
        ])
        .split(inner);

    // Progress gauge
    let gauge_label = format!("{}/{} ({:.0}%)", passing, total, progress_ratio * 100.0);
    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(theme.done))
        .label(gauge_label)
        .ratio(progress_ratio);
    frame.render_widget(gauge, chunks[0]);

    // Agent counts (placeholder)
    let agent_info = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            format!("Coding Agents: {in_progress}/5     "),
            Style::default().fg(theme.in_progress),
        ),
        Span::styled(
            "Testing Agents: 1/5".to_string(),
            Style::default().fg(theme.secondary),
        ),
    ])])
    .alignment(Alignment::Left);
    frame.render_widget(agent_info, chunks[2]);

    // Recent activity
    let activity = Paragraph::new(vec![
        Line::from(Span::styled(
            "Recent Activity:",
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.done)),
            Span::styled(
                format!(
                    "{} features completed",
                    app.features.iter().filter(|f| f.passes).count()
                ),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.in_progress)),
            Span::styled(
                format!(
                    "{} features in progress",
                    app.features.iter().filter(|f| f.in_progress).count()
                ),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.pending)),
            Span::styled(
                format!(
                    "{} features pending",
                    app.features
                        .iter()
                        .filter(|f| !f.passes && !f.in_progress)
                        .count()
                ),
                Style::default().fg(theme.foreground),
            ),
        ]),
    ])
    .alignment(Alignment::Left);
    frame.render_widget(activity, chunks[4]);

    // Render footer with key hints
    let footer_text = if let Some(status) = app.get_status_message() {
        status.to_string()
    } else {
        " 1-4:views  r:refresh  t:theme  ?:help  q:quit".to_string()
    };

    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.muted));
    frame.render_widget(footer, footer_area);
}
