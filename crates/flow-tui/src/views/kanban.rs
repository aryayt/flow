use crate::app::App;
use flow_core::Feature;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Split into three columns for Kanban board
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    // Separate features by status
    let pending: Vec<&Feature> = app
        .features
        .iter()
        .filter(|f| !f.passes && !f.in_progress)
        .collect();

    let in_progress: Vec<&Feature> = app
        .features
        .iter()
        .filter(|f| f.in_progress && !f.passes)
        .collect();

    let done: Vec<&Feature> = app.features.iter().filter(|f| f.passes).collect();

    // Render each column
    render_column(frame, columns[0], "Pending", &pending, app, "○");
    render_column(frame, columns[1], "In Progress", &in_progress, app, "◉");
    render_column(frame, columns[2], "Done", &done, app, "✓");
}

fn render_column(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    features: &[&Feature],
    app: &App,
    status_icon: &str,
) {
    let theme = &app.tui_theme;

    // Determine color based on column
    let title_color = match title {
        "Pending" => theme.pending,
        "In Progress" => theme.in_progress,
        "Done" => theme.done,
        _ => theme.foreground,
    };

    let block = Block::default()
        .title(format!("{} ({})", title, features.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if features.is_empty() {
        let empty = Paragraph::new("(empty)")
            .style(Style::default().fg(theme.muted))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Create list items for features
    let items: Vec<ListItem> = features
        .iter()
        .map(|feature| {
            let is_selected = app
                .features
                .iter()
                .position(|f| f.id == feature.id)
                .is_some_and(|pos| pos == app.selected_index);

            let status_color = if !feature.passes && !feature.in_progress {
                theme.pending
            } else if feature.in_progress {
                theme.in_progress
            } else {
                theme.done
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(format!("{status_icon} "), Style::default().fg(status_color)),
                    Span::styled(
                        &feature.name,
                        Style::default()
                            .fg(if is_selected {
                                theme.primary
                            } else {
                                theme.foreground
                            })
                            .add_modifier(if is_selected {
                                Modifier::BOLD | Modifier::REVERSED
                            } else {
                                Modifier::empty()
                            }),
                    ),
                ]),
                Line::from(Span::styled(
                    format!("  {}", &feature.category),
                    Style::default().fg(theme.secondary),
                )),
            ];

            // Show agent for in-progress items (placeholder)
            if feature.in_progress {
                lines.push(Line::from(Span::styled(
                    "  [Agent: Working]",
                    Style::default().fg(theme.accent),
                )));
            }

            // Show if blocked (has unsatisfied dependencies)
            if !feature.dependencies.is_empty() {
                let satisfied = feature.dependencies.iter().all(|dep_id| {
                    app.features
                        .iter()
                        .find(|f| f.id == *dep_id)
                        .is_some_and(|f| f.passes)
                });
                if !satisfied {
                    lines.push(Line::from(Span::styled(
                        "  [Blocked]",
                        Style::default().fg(theme.blocked),
                    )));
                }
            }

            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}
