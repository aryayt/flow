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

    // Split main area into graph and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Graph
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let graph_area = chunks[0];
    let footer_area = chunks[1];

    let block = Block::default()
        .title("Dependency Graph")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(graph_area);
    frame.render_widget(block, graph_area);

    if app.features.is_empty() {
        return;
    }

    // Create a simple vertical layout showing features and their dependencies
    let mut items = Vec::new();

    for feature in &app.features {
        // Determine status color
        let status_color = if feature.passes {
            theme.done
        } else if feature.in_progress {
            theme.in_progress
        } else {
            theme.pending
        };

        // Status icon
        let icon = if feature.passes {
            "✓"
        } else if feature.in_progress {
            "◉"
        } else {
            "○"
        };

        // Check if blocked
        let is_blocked = !feature.dependencies.is_empty()
            && feature.dependencies.iter().any(|dep_id| {
                app.features
                    .iter()
                    .find(|f| f.id == *dep_id)
                    .is_none_or(|f| !f.passes)
            });

        let border_color = if is_blocked {
            theme.blocked
        } else {
            status_color
        };

        // Create feature box
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("┌─", Style::default().fg(border_color)),
                Span::styled(
                    format!(" {} {} ", icon, feature.name),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("─┐", Style::default().fg(border_color)),
            ]),
            Line::from(vec![
                Span::styled("│ ", Style::default().fg(border_color)),
                Span::styled(&feature.category, Style::default().fg(theme.secondary)),
                Span::styled(" │", Style::default().fg(border_color)),
            ]),
            Line::from(vec![Span::styled(
                "└──────────────────┘",
                Style::default().fg(border_color),
            )]),
        ]));

        // Show dependencies with arrows
        if !feature.dependencies.is_empty() {
            for dep_id in &feature.dependencies {
                if let Some(dep) = app.features.iter().find(|f| f.id == *dep_id) {
                    let dep_status = if dep.passes {
                        "✓"
                    } else if dep.in_progress {
                        "◉"
                    } else {
                        "○"
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::styled("  ↓ ", Style::default().fg(theme.accent)),
                        Span::styled(
                            format!("{dep_status} depends on: "),
                            Style::default().fg(theme.muted),
                        ),
                        Span::styled(&dep.name, Style::default().fg(theme.foreground)),
                    ])));
                }
            }
        }

        items.push(ListItem::new(Line::from("")));
    }

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
