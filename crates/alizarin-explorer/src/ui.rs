//! UI rendering

use crate::app::{App, GraphsView, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.current_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(Span::styled(t.title(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Alizarin Explorer "),
        )
        .select(app.current_tab.index())
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Indexing => draw_indexing_tab(f, app, area),
        Tab::Graphs => draw_graphs_tab(f, app, area),
        Tab::BusinessData => draw_business_data_tab(f, app, area),
        Tab::ControlledVocabularies => draw_cv_tab(f, app, area),
    }
}

fn draw_indexing_tab(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(format!("Prebuild path: {}", app.info.path.display())),
        Line::from(""),
        Line::from(format!(
            "Index Templates: {}",
            if app.info.has_index_templates {
                "Yes"
            } else {
                "No"
            }
        )),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Indexing "),
    );

    f.render_widget(paragraph, area);
}

fn draw_graphs_tab(f: &mut Frame, app: &App, area: Rect) {
    match app.graphs_view {
        GraphsView::List => draw_graphs_list(f, app, area),
        GraphsView::Tree => draw_graph_tree(f, app, area),
    }
}

fn draw_graphs_list(f: &mut Frame, app: &App, area: Rect) {
    if !app.graphs_loaded {
        let paragraph = Paragraph::new("Loading graphs...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Graphs "),
        );
        f.render_widget(paragraph, area);
        return;
    }

    if app.graphs.is_empty() {
        let paragraph = Paragraph::new("No graphs found").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Graphs "),
        );
        f.render_widget(paragraph, area);
        return;
    }

    let header_cells = ["Name", "UUID", "Subtitle", "Author"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .graphs
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let style = if i == app.graph_list_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let cells = vec![
                Cell::from(g.graph.display_name()),
                Cell::from(truncate(&g.graph.graphid, 36)),
                Cell::from(truncate(&g.graph.display_subtitle(), 30)),
                Cell::from(truncate(&g.graph.display_author(), 20)),
            ];
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Length(38),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Graphs (Enter to explore, Tab to switch) "),
    );

    f.render_widget(table, area);
}

fn draw_graph_tree(f: &mut Frame, app: &App, area: Rect) {
    let title = if let Some(g) = app.selected_graph() {
        format!(" {} - Tree View (Esc to go back) ", g.graph.display_name())
    } else {
        " Tree View ".to_string()
    };

    if app.tree_nodes.is_empty() {
        let paragraph = Paragraph::new("No nodes").block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        );
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible area (accounting for borders)
    let visible_height = area.height.saturating_sub(2) as usize;

    // Adjust scroll offset to keep selected item visible
    let scroll_offset = calculate_scroll_offset(
        app.tree_selected,
        app.tree_scroll_offset,
        visible_height,
        app.tree_nodes.len(),
    );

    let lines: Vec<Line> = app
        .tree_nodes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, node)| {
            let is_selected = i == app.tree_selected;

            // Build the tree prefix with proper indentation
            let indent = "  ".repeat(node.depth);
            let expand_char = if node.has_children {
                if node.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let line_content = format!(
                "{}{}{} [{}]{}",
                indent,
                expand_char,
                node.name,
                node.datatype,
                if node.alias.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", node.alias)
                }
            );

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Line::from(Span::styled(line_content, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );

    f.render_widget(paragraph, area);
}

fn draw_business_data_tab(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.info.has_business_data {
        "Business data directory found"
    } else {
        "No business data directory"
    };

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Business Data "),
    );

    f.render_widget(paragraph, area);
}

fn draw_cv_tab(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.info.has_reference_data {
        "Reference data directory found"
    } else {
        "No reference data directory"
    };

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Controlled Vocabularies "),
    );

    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.current_tab {
        Tab::Graphs => match app.graphs_view {
            GraphsView::List => "↑/↓: Navigate | Enter: Explore | Tab: Next tab | q: Quit",
            GraphsView::Tree => "↑/↓: Navigate | →: Expand | ←: Collapse | Esc: Back | q: Quit",
        },
        _ => "Tab: Next tab | q: Quit",
    };

    let status = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));

    f.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

fn calculate_scroll_offset(
    selected: usize,
    current_offset: usize,
    visible_height: usize,
    total_items: usize,
) -> usize {
    if visible_height >= total_items {
        return 0;
    }

    let mut offset = current_offset;

    // If selected is above visible area, scroll up
    if selected < offset {
        offset = selected;
    }

    // If selected is below visible area, scroll down
    if selected >= offset + visible_height {
        offset = selected - visible_height + 1;
    }

    // Don't scroll past the end
    offset.min(total_items.saturating_sub(visible_height))
}
