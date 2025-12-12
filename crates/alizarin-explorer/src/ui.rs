//! UI rendering

use crate::app::{App, BusinessDataSource, BusinessDataView, GraphsView, Tab, TreeNode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Tabs},
    Frame,
};
use std::collections::HashMap;

/// Colors for nodegroups - cycle through these for different nodegroups
const NODEGROUP_COLORS: &[Color] = &[
    Color::Rgb(60, 60, 90),    // Dark blue
    Color::Rgb(60, 90, 60),    // Dark green
    Color::Rgb(90, 60, 60),    // Dark red
    Color::Rgb(90, 90, 60),    // Dark yellow
    Color::Rgb(60, 90, 90),    // Dark cyan
    Color::Rgb(90, 60, 90),    // Dark magenta
    Color::Rgb(70, 70, 70),    // Dark gray
    Color::Rgb(80, 60, 50),    // Dark brown
];

pub fn draw(f: &mut Frame, app: &mut App) {
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

fn draw_tabs(f: &mut Frame, app: &mut App, area: Rect) {
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

fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    match app.current_tab {
        Tab::Indexing => draw_indexing_tab(f, app, area),
        Tab::Graphs => draw_graphs_tab(f, app, area),
        Tab::BusinessData => draw_business_data_tab(f, app, area),
        Tab::ControlledVocabularies => draw_cv_tab(f, app, area),
    }
}

fn draw_indexing_tab(f: &mut Frame, app: &mut App, area: Rect) {
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

fn draw_graphs_tab(f: &mut Frame, app: &mut App, area: Rect) {
    match app.graphs_view {
        GraphsView::List => draw_graphs_list(f, app, area),
        GraphsView::Tree => draw_graph_tree(f, app, area),
    }
}

fn draw_graphs_list(f: &mut Frame, app: &mut App, area: Rect) {
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

fn draw_graph_tree(f: &mut Frame, app: &mut App, area: Rect) {
    // If in search mode, add a search input bar at the top
    let (main_area, search_area) = if app.search_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Split horizontally: 2/3 for tree, 1/3 for detail pane
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
        .split(main_area);

    draw_tree_pane(f, app, chunks[0]);
    draw_detail_pane(f, app, chunks[1]);

    // Draw search input if in search mode
    if let Some(search_area) = search_area {
        draw_search_input(f, app, search_area);
    }
}

fn draw_search_input(f: &mut Frame, app: &mut App, area: Rect) {
    let match_count = app.search_match_count();
    let case_indicator = if app.search_case_sensitive { "[Aa]" } else { "[a]" };
    let title = if let Some(ref err) = app.search_error {
        format!(" Search {} (error: {}) ", case_indicator, err)
    } else if match_count > 0 {
        format!(" Search {} ({} matches) [Ctrl+i: toggle case] ", case_indicator, match_count)
    } else if !app.search_query.is_empty() {
        format!(" Search {} (no matches) ", case_indicator)
    } else {
        format!(" Search {} (regex) [Ctrl+i: toggle case] ", case_indicator)
    };

    let search_text = format!("/{}", app.search_query);

    let style = if app.search_error.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let paragraph = Paragraph::new(search_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(paragraph, area);
}

/// Build a color map for nodegroups
fn build_nodegroup_color_map(app: &mut App) -> HashMap<String, Color> {
    let mut map = HashMap::new();
    let nodegroup_ids = app.visible_nodegroup_ids();

    for (i, id) in nodegroup_ids.iter().enumerate() {
        map.insert(id.clone(), NODEGROUP_COLORS[i % NODEGROUP_COLORS.len()]);
    }

    map
}

/// Get the background color for a tree node based on nodegroup highlighting
fn get_node_bg_color(
    node: &TreeNode,
    is_selected: bool,
    selected_nodegroup_id: Option<&String>,
    nodegroup_colors: &HashMap<String, Color>,
) -> Option<Color> {
    if is_selected {
        return Some(Color::White);
    }

    // If this node shares nodegroup with selected node, highlight it
    if let (Some(node_ng), Some(selected_ng)) = (&node.nodegroup_id, selected_nodegroup_id) {
        if node_ng == selected_ng {
            return nodegroup_colors.get(node_ng).copied();
        }
    }

    None
}

/// Build a color map for nodegroups in tile tree
fn build_tile_nodegroup_color_map(app: &mut App) -> HashMap<String, Color> {
    let mut map = HashMap::new();
    let nodegroup_ids = app.bd_tile_visible_nodegroup_ids();

    for (i, id) in nodegroup_ids.iter().enumerate() {
        map.insert(id.clone(), NODEGROUP_COLORS[i % NODEGROUP_COLORS.len()]);
    }

    map
}

/// Get the background color for a tile tree node based on nodegroup highlighting
fn get_tile_node_bg_color(
    node: &crate::app::TileTreeNode,
    selected_nodegroup_id: Option<&String>,
    nodegroup_colors: &HashMap<String, Color>,
) -> Option<Color> {
    // If this node shares nodegroup with selected node, highlight it
    if let (Some(node_ng), Some(selected_ng)) = (&node.nodegroup_id, selected_nodegroup_id) {
        if node_ng == selected_ng {
            return nodegroup_colors.get(node_ng).copied();
        }
    }

    None
}

fn draw_tree_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let title = if app.search_mode {
        if let Some(g) = app.selected_graph() {
            format!(" {} (searching...) ", g.graph.display_name())
        } else {
            " Tree View (searching...) ".to_string()
        }
    } else if let Some(g) = app.selected_graph() {
        format!(" {} (Esc to go back, / to search) ", g.graph.display_name())
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

    // Get selected node's nodegroup for highlighting (clone to avoid borrow conflict)
    let selected_nodegroup_id: Option<String> = app
        .selected_tree_node()
        .and_then(|n| n.nodegroup_id.clone());

    let nodegroup_colors = build_nodegroup_color_map(app);

    // Filter nodes if search is active (clone to avoid borrow conflict with scroll offset update)
    let is_searching = app.search_mode || !app.search_query.is_empty();
    let tree_selected = app.tree_selected;
    let visible_nodes: Vec<(usize, TreeNode)> = app
        .tree_nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !is_searching || node.visible_in_search)
        .map(|(i, node)| (i, node.clone()))
        .collect();

    // Calculate visible area (accounting for borders)
    let visible_height = area.height.saturating_sub(2) as usize;

    // Find position of selected node in filtered list
    let selected_pos_in_filtered = visible_nodes
        .iter()
        .position(|(i, _)| *i == tree_selected)
        .unwrap_or(0);

    // Adjust scroll offset to keep selected item visible (using persisted offset)
    let scroll_offset = calculate_scroll_offset(
        selected_pos_in_filtered,
        app.tree_scroll_offset,
        visible_height,
        visible_nodes.len(),
    );
    app.tree_scroll_offset = scroll_offset;

    let lines: Vec<Line> = visible_nodes
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, node)| {
            let is_selected = *i == app.tree_selected;

            // Build the tree prefix with proper indentation
            let indent = "  ".repeat(node.depth);
            let expand_char = if node.has_children {
                if node.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            // Node indicators
            let collector_char = if node.is_collector { "◉" } else { "○" };
            let required_char = if node.is_required { "*" } else { "" };

            // Format: alias (if present), then name, then [type]
            let line_content = if node.alias.is_empty() {
                format!(
                    "{}{}{} {} [{}]{}",
                    indent,
                    expand_char,
                    collector_char,
                    node.name,
                    node.datatype,
                    required_char
                )
            } else {
                format!(
                    "{}{}{} {} ({}) [{}]{}",
                    indent,
                    expand_char,
                    collector_char,
                    node.alias,
                    node.name,
                    node.datatype,
                    required_char
                )
            };

            // Determine style based on selection and search match status
            let style = if is_selected {
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if node.matches_search {
                // Highlight search matches with bright color
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
            } else if let Some(bg) = get_node_bg_color(node, false, selected_nodegroup_id.as_ref(), &nodegroup_colors) {
                Style::default().bg(bg)
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

fn draw_detail_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let selected = app.selected_tree_node();
    let nodegroup = app.selected_nodegroup();

    let mut lines: Vec<Line> = Vec::new();

    // Node Information Section
    lines.push(Line::from(Span::styled(
        "─── Node ───",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(node) = selected {
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(&node.name),
        ]));

        if !node.alias.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Alias: ", Style::default().fg(Color::Cyan)),
                Span::raw(&node.alias),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(&node.datatype),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Node ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(&node.node_id, Style::default().fg(Color::DarkGray)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Collector: ", Style::default().fg(Color::Cyan)),
            Span::raw(if node.is_collector { "Yes ◉" } else { "No" }),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Required: ", Style::default().fg(Color::Cyan)),
            Span::raw(if node.is_required { "Yes *" } else { "No" }),
        ]));

        if let Some(ref onto) = node.ontologyclass {
            lines.push(Line::from(vec![
                Span::styled("Ontology: ", Style::default().fg(Color::Cyan)),
                Span::styled(onto, Style::default().fg(Color::DarkGray)),
            ]));
        }

        if let Some(ref desc) = node.description {
            if !desc.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("Description:", Style::default().fg(Color::Cyan))));
                // Wrap description text
                for line in wrap_text(desc, area.width.saturating_sub(4) as usize) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(Color::DarkGray))));
                }
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No node selected",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Nodegroup Information Section
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── Nodegroup ───",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(ng) = nodegroup {
        lines.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(&ng.nodegroupid, Style::default().fg(Color::DarkGray)),
        ]));

        if let Some(ref card) = ng.cardinality {
            lines.push(Line::from(vec![
                Span::styled("Cardinality: ", Style::default().fg(Color::Cyan)),
                Span::raw(card),
            ]));
        }

        if let Some(ref parent) = ng.parentnodegroup_id {
            lines.push(Line::from(vec![
                Span::styled("Parent: ", Style::default().fg(Color::Cyan)),
                Span::styled(parent, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Show count of nodes sharing this nodegroup
        let shared_count = app
            .tree_nodes
            .iter()
            .filter(|n| n.nodegroup_id.as_ref() == Some(&ng.nodegroupid))
            .count();

        if shared_count > 1 {
            lines.push(Line::from(vec![
                Span::styled("Shared by: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} nodes (highlighted)", shared_count),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
    } else if selected.is_some() {
        lines.push(Line::from(Span::styled(
            "Root node (no nodegroup)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "No nodegroup",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Legend
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── Legend ───",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("◉ = Collector  "),
        Span::raw("* = Required"),
    ]));
    lines.push(Line::from(Span::styled(
        "Highlighted = Same nodegroup",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details "),
    );

    f.render_widget(paragraph, area);
}

/// Wrap text to a given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn draw_business_data_tab(f: &mut Frame, app: &mut App, area: Rect) {
    if !app.info.has_business_data {
        let paragraph = Paragraph::new("No business data directory").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Business Data "),
        );
        f.render_widget(paragraph, area);
        return;
    }

    match app.bd_view {
        BusinessDataView::GraphList => draw_bd_graph_list(f, app, area),
        BusinessDataView::ResourceList => draw_bd_resource_list(f, app, area),
        BusinessDataView::ResourceDetail => draw_bd_resource_detail(f, app, area),
    }
}

fn draw_bd_graph_list(f: &mut Frame, app: &mut App, area: Rect) {
    if app.graphs.is_empty() {
        let paragraph = Paragraph::new("No graphs found").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Business Data - Select Graph "),
        );
        f.render_widget(paragraph, area);
        return;
    }

    let header_cells = ["Name", "UUID"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .graphs
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let style = if i == app.bd_graph_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let cells = vec![
                Cell::from(g.graph.display_name()),
                Cell::from(g.graph.graphid.clone()),
            ];
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Business Data - Select Graph (Enter to explore) "),
    );

    f.render_widget(table, area);
}

fn draw_bd_resource_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Split for search bar at the top if in search mode or has a query
    let (main_area, search_area) = if app.bd_search_mode || !app.bd_search_query.is_empty() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        (chunks[1], Some(chunks[0]))
    } else {
        (area, None)
    };

    // Split horizontally: 2/3 for list, 1/3 for detail pane
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
        .split(main_area);

    draw_bd_list_pane(f, app, chunks[0]);
    draw_bd_detail_pane(f, app, chunks[1]);

    // Draw search input
    if let Some(search_area) = search_area {
        draw_bd_search_input(f, app, search_area);
    }
}

fn draw_bd_search_input(f: &mut Frame, app: &mut App, area: Rect) {
    let filter_count = app.bd_display_count();
    let total = app.bd_resources.len();
    let case_indicator = if app.bd_search_case_sensitive { "[Aa]" } else { "[a]" };

    let title = if !app.bd_search_query.is_empty() {
        format!(" Filter {} ({}/{}) ", case_indicator, filter_count, total)
    } else {
        format!(" Filter {} (regex) [Ctrl+i: toggle case] ", case_indicator)
    };

    let search_text = format!("/{}", app.bd_search_query);

    let style = Style::default().fg(Color::Yellow);

    let paragraph = Paragraph::new(search_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(paragraph, area);
}

fn draw_bd_list_pane(f: &mut Frame, app: &App, area: Rect) {
    let graph_name = app
        .bd_selected_graph()
        .map(|g| g.graph.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    // Build title
    let progress = if let Some(total) = app.bd_total_count {
        format!(" ({}/{}) ", app.bd_loaded_count, total)
    } else {
        format!(" ({}) ", app.bd_loaded_count)
    };

    let title = format!(
        " {} - Resources{} (Esc to go back, / to filter) ",
        graph_name, progress
    );

    let resources = app.bd_display_resources();

    // Show progress bar when loading
    if app.bd_loading {
        let (list_area, progress_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(area);
            (chunks[0], chunks[1])
        };

        // Draw the list in the main area
        draw_bd_list_content(f, app, &resources, list_area, &title);

        // Draw progress bar
        let progress_ratio = if let Some(total) = app.bd_total_count {
            if total > 0 {
                app.bd_loaded_count as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        let progress_label = if let Some(total) = app.bd_total_count {
            // We have the total - show loading progress
            format!("Loading... {}/{} ({:.0}%)", app.bd_loaded_count, total, progress_ratio * 100.0)
        } else if app.bd_counting_files > 0 {
            // Still counting - show counting progress
            format!("Counting... {} files, {} resources found", app.bd_counting_files, app.bd_counting_resources)
        } else {
            // Just started
            "Starting...".to_string()
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Loading "))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .ratio(progress_ratio.min(1.0))
            .label(progress_label);

        f.render_widget(gauge, progress_area);
        return;
    }

    if resources.is_empty() {
        let msg = if !app.bd_search_query.is_empty() {
            "No resources match filter"
        } else {
            "No resources found"
        };
        let paragraph = Paragraph::new(msg).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        );
        f.render_widget(paragraph, area);
        return;
    }

    draw_bd_list_content(f, app, &resources, area, &title);
}

fn draw_bd_list_content(f: &mut Frame, app: &App, resources: &[&alizarin_core::StaticResourceSummary], area: Rect, title: &str) {
    if resources.is_empty() {
        let paragraph = Paragraph::new("Loading...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.to_string()),
        );
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible area (accounting for borders)
    let visible_height = area.height.saturating_sub(2) as usize;
    let available_width = area.width.saturating_sub(2) as usize; // subtract borders

    // Calculate scroll offset
    let scroll_offset = calculate_scroll_offset(
        app.bd_resource_selected,
        0,
        visible_height,
        resources.len(),
    );

    let lines: Vec<Line> = resources
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, resource)| {
            let is_selected = i == app.bd_resource_selected;

            let style = if is_selected {
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Format: name [resourceinstanceid]
            // UUID is 36 chars, overhead is 3 chars (" [" and "]")
            let uuid_len = resource.resourceinstanceid.len();
            let name_len = resource.name.chars().count();
            let overhead = 3; // " [" + "]"
            let full_len = name_len + overhead + uuid_len;

            let display = if full_len <= available_width {
                // Show full UUID
                format!("{} [{}]", resource.name, resource.resourceinstanceid)
            } else {
                // Calculate how much space we have for UUID after name
                let uuid_space = available_width.saturating_sub(name_len + overhead);
                if uuid_space >= 8 {
                    // Show truncated UUID
                    format!("{} [{}]", resource.name, truncate(&resource.resourceinstanceid, uuid_space))
                } else {
                    // Not enough space, just show name
                    truncate(&resource.name, available_width)
                }
            };

            Line::from(Span::styled(display, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );

    f.render_widget(paragraph, area);
}

fn draw_bd_detail_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Resource Information Section
    lines.push(Line::from(Span::styled(
        "─── Resource ───",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(resource) = app.bd_selected_resource() {
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(&resource.name),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Resource ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(&resource.resourceinstanceid, Style::default().fg(Color::DarkGray)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Graph ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(&resource.graph_id, Style::default().fg(Color::DarkGray)),
        ]));

        if let Some(ref created) = resource.createdtime {
            lines.push(Line::from(vec![
                Span::styled("Created: ", Style::default().fg(Color::Cyan)),
                Span::raw(created),
            ]));
        }

        if let Some(ref modified) = resource.lastmodified {
            lines.push(Line::from(vec![
                Span::styled("Modified: ", Style::default().fg(Color::Cyan)),
                Span::raw(modified),
            ]));
        }

        if let Some(ref pub_id) = resource.publication_id {
            lines.push(Line::from(vec![
                Span::styled("Publication ID: ", Style::default().fg(Color::Cyan)),
                Span::styled(pub_id, Style::default().fg(Color::DarkGray)),
            ]));
        }

        if let Some(ref legacy) = resource.legacyid {
            if !legacy.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Legacy ID: ", Style::default().fg(Color::Cyan)),
                    Span::raw(legacy),
                ]));
            }
        }

        if let Some(user_id) = resource.principaluser_id {
            lines.push(Line::from(vec![
                Span::styled("User ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(user_id.to_string()),
            ]));
        }

        // Descriptors section
        if let Some(ref descriptors) = resource.descriptors {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─── Descriptors ───",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            let desc_name = descriptors.name.as_deref().unwrap_or_default();
            if !desc_name.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Display Name: ", Style::default().fg(Color::Cyan)),
                    Span::raw(desc_name),
                ]));
            }

            let desc_text = descriptors.description.as_deref().unwrap_or_default();
            if !desc_text.is_empty() {
                lines.push(Line::from(Span::styled("Description:", Style::default().fg(Color::Cyan))));
                for line in wrap_text(desc_text, area.width.saturating_sub(4) as usize) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(Color::DarkGray))));
                }
            }
        }

        // Metadata section (HashMap<String, String>)
        if !resource.metadata.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─── Metadata ───",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for (key, value) in &resource.metadata {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(Color::Cyan)),
                    Span::raw(value),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No resource selected",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details "),
    );

    f.render_widget(paragraph, area);
}

fn draw_bd_resource_detail(f: &mut Frame, app: &mut App, area: Rect) {
    // Split horizontally: 1/3 for resource list, 2/3 for detail view
    let horiz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(area);

    // Left pane: narrower resource list
    draw_bd_resource_list_narrow(f, app, horiz_chunks[0]);

    // Right pane: split vertically - 2/3 for tile tree, 1/3 for tile info
    let vert_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
        .split(horiz_chunks[1]);

    draw_bd_tile_tree(f, app, vert_chunks[0]);
    draw_bd_tile_info(f, app, vert_chunks[1]);
}

fn draw_bd_resource_list_narrow(f: &mut Frame, app: &mut App, area: Rect) {
    let graph_name = app
        .bd_selected_graph()
        .map(|g| g.graph.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let title = format!(" {} Resources ", graph_name);

    let resources = app.bd_display_resources();

    if resources.is_empty() {
        let paragraph = Paragraph::new("No resources").block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        );
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible area
    let visible_height = area.height.saturating_sub(2) as usize;

    // Calculate scroll offset
    let scroll_offset = calculate_scroll_offset(
        app.bd_resource_selected,
        0,
        visible_height,
        resources.len(),
    );

    let lines: Vec<Line> = resources
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, resource)| {
            let is_selected = i == app.bd_resource_selected;

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Just show name truncated to fit
            let max_len = area.width.saturating_sub(4) as usize;
            Line::from(Span::styled(truncate(&resource.name, max_len), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );

    f.render_widget(paragraph, area);
}

fn draw_bd_tile_tree(f: &mut Frame, app: &mut App, area: Rect) {
    // If in search mode, add a search input bar at the bottom
    let (main_area, search_area) = if app.bd_tile_search_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let title = if app.bd_tile_search_mode {
        " Tile Tree (searching...) "
    } else if app.bd_resource_loading {
        " Loading... "
    } else if app.bd_current_resource.is_some() {
        " Tile Tree (/ to search) "
    } else {
        " No Resource Loaded "
    };

    if app.bd_tile_tree.is_empty() {
        let msg = if app.bd_resource_loading {
            "Loading resource..."
        } else {
            "No tiles"
        };
        let paragraph = Paragraph::new(msg).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        );
        f.render_widget(paragraph, main_area);
        return;
    }

    // Get selected node's nodegroup for highlighting (clone to avoid borrow conflict)
    let selected_nodegroup_id: Option<String> = app
        .bd_selected_tile_node()
        .and_then(|n| n.nodegroup_id.clone());

    // Build nodegroup color map for tile tree
    let tile_nodegroup_colors = build_tile_nodegroup_color_map(app);

    // Filter nodes if search is active (clone nodes to avoid borrow conflict with scroll offset update)
    let is_searching = app.bd_tile_search_mode || !app.bd_tile_search_query.is_empty();
    let bd_tile_selected = app.bd_tile_selected;
    let visible_nodes: Vec<(usize, crate::app::TileTreeNode)> = app
        .bd_tile_tree
        .iter()
        .enumerate()
        .filter(|(_, node)| !is_searching || node.visible_in_search)
        .map(|(i, node)| (i, node.clone()))
        .collect();

    // Calculate visible area (accounting for borders)
    let visible_height = main_area.height.saturating_sub(2) as usize;

    // Find position of selected node in filtered list
    let selected_pos_in_filtered = visible_nodes
        .iter()
        .position(|(i, _)| *i == bd_tile_selected)
        .unwrap_or(0);

    // Adjust scroll offset to keep selected item visible (using persisted offset)
    let scroll_offset = calculate_scroll_offset(
        selected_pos_in_filtered,
        app.bd_tile_scroll_offset,
        visible_height,
        visible_nodes.len(),
    );
    app.bd_tile_scroll_offset = scroll_offset;

    // Build a map of nodegroup_id -> list of indices in visible_nodes for bracket rendering
    let mut nodegroup_positions: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for (pos, (_, node)) in visible_nodes.iter().enumerate() {
        if let Some(ref ng_id) = node.nodegroup_id {
            nodegroup_positions
                .entry(ng_id.clone())
                .or_default()
                .push(pos);
        }
    }

    // Only show brackets for nodegroups with multiple tiles
    let multi_tile_nodegroups: std::collections::HashSet<String> = nodegroup_positions
        .iter()
        .filter(|(_, positions)| positions.len() > 1)
        .map(|(id, _)| id.clone())
        .collect();

    let lines: Vec<Line> = visible_nodes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(pos, (i, node))| {
            let is_selected = *i == app.bd_tile_selected;

            // Determine bracket character for grouped nodes
            let bracket = if let Some(ref ng_id) = node.nodegroup_id {
                if multi_tile_nodegroups.contains(ng_id) {
                    if let Some(positions) = nodegroup_positions.get(ng_id) {
                        let first = positions.first() == Some(&pos);
                        let last = positions.last() == Some(&pos);
                        if first && last {
                            "─" // Single item (shouldn't happen, but handle it)
                        } else if first {
                            "┌"
                        } else if last {
                            "└"
                        } else {
                            "│"
                        }
                    } else {
                        " "
                    }
                } else {
                    " "
                }
            } else {
                " "
            };

            // Build the tree prefix with proper indentation
            let indent = "  ".repeat(node.depth);
            let expand_char = if node.has_children {
                if node.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            // Format: bracket + indent + expand + name + [type]
            let datatype = node.datatype.as_deref().unwrap_or("?");
            let line_content = format!("{}{}{}{} [{}]", bracket, indent, expand_char, node.name, datatype);

            // Determine style based on selection, search match, and nodegroup
            let style = if is_selected {
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if node.matches_search {
                // Highlight search matches with bright color
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
            } else if let Some(bg) = get_tile_node_bg_color(node, selected_nodegroup_id.as_ref(), &tile_nodegroup_colors) {
                // Highlight nodes sharing nodegroup with selected node
                Style::default().bg(bg)
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

    f.render_widget(paragraph, main_area);

    // Draw search input if in search mode
    if let Some(search_area) = search_area {
        draw_bd_tile_search_input(f, app, search_area);
    }
}

fn draw_bd_tile_search_input(f: &mut Frame, app: &mut App, area: Rect) {
    let match_count = app.bd_tile_search_match_count();
    let case_indicator = if app.bd_tile_search_case_sensitive { "[Aa]" } else { "[a]" };
    let title = if let Some(ref err) = app.bd_tile_search_error {
        format!(" Search {} (error: {}) ", case_indicator, err)
    } else if match_count > 0 {
        format!(" Search {} ({} matches) [Ctrl+i: toggle case] ", case_indicator, match_count)
    } else if !app.bd_tile_search_query.is_empty() {
        format!(" Search {} (no matches) ", case_indicator)
    } else {
        format!(" Search {} (regex) [Ctrl+i: toggle case] ", case_indicator)
    };

    let search_text = format!("/{}", app.bd_tile_search_query);

    let style = if app.bd_tile_search_error.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let paragraph = Paragraph::new(search_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(paragraph, area);
}

fn draw_bd_tile_info(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Available width for content (subtract borders)
    let available_width = area.width.saturating_sub(2) as usize;

    lines.push(Line::from(Span::styled(
        "─── Tile Info ───",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(node) = app.bd_selected_tile_node() {
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(&node.name),
        ]));

        if let Some(ref datatype) = node.datatype {
            lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                Span::raw(datatype),
            ]));
        }

        // Helper to show UUID with dynamic truncation based on available space
        let format_uuid = |label: &str, uuid: &str| -> Line {
            let label_len = label.len();
            let uuid_len = uuid.len();
            let uuid_display = if label_len + uuid_len <= available_width {
                uuid.to_string()
            } else {
                truncate(uuid, available_width.saturating_sub(label_len))
            };
            Line::from(vec![
                Span::styled(label.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(uuid_display, Style::default().fg(Color::DarkGray)),
            ])
        };

        if let Some(ref node_id) = node.node_id {
            lines.push(format_uuid("Node ID: ", node_id));
        }

        if let Some(ref tile_id) = node.tile_id {
            lines.push(format_uuid("Tile ID: ", tile_id));
        }

        if let Some(ref nodegroup_id) = node.nodegroup_id {
            lines.push(format_uuid("Nodegroup: ", nodegroup_id));

            // Show count of tiles sharing this nodegroup
            let shared_count = app
                .bd_tile_tree
                .iter()
                .filter(|n| n.nodegroup_id.as_ref() == Some(nodegroup_id))
                .count();

            if shared_count > 1 {
                lines.push(Line::from(vec![
                    Span::styled("Shared by: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{} tiles (highlighted)", shared_count),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }
        }

        // Show value if present
        if let Some(ref value) = node.value {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─── Value ───",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Format the JSON value nicely
            let value_str = format_json_value(value, area.width.saturating_sub(4) as usize);
            for line in value_str {
                lines.push(Line::from(Span::styled(line, Style::default().fg(Color::Green))));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No tile selected",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details "),
    );

    f.render_widget(paragraph, area);
}

/// Format a JSON value for display
fn format_json_value(value: &serde_json::Value, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    match value {
        serde_json::Value::Null => {
            lines.push("null".to_string());
        }
        serde_json::Value::Bool(b) => {
            lines.push(b.to_string());
        }
        serde_json::Value::Number(n) => {
            lines.push(n.to_string());
        }
        serde_json::Value::String(s) => {
            // Wrap long strings
            for line in wrap_text(s, max_width) {
                lines.push(line);
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                lines.push("[]".to_string());
            } else {
                lines.push(format!("[{} items]", arr.len()));
                for (i, item) in arr.iter().take(5).enumerate() {
                    let formatted = serde_json::to_string(item).unwrap_or_else(|_| "?".to_string());
                    let truncated = if formatted.len() > max_width {
                        format!("{}...", &formatted[..max_width.saturating_sub(3)])
                    } else {
                        formatted
                    };
                    lines.push(format!("  [{}]: {}", i, truncated));
                }
                if arr.len() > 5 {
                    lines.push(format!("  ... ({} more)", arr.len() - 5));
                }
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                lines.push("{}".to_string());
            } else {
                for (key, val) in obj.iter().take(10) {
                    let val_str = match val {
                        serde_json::Value::String(s) => {
                            if s.len() > 30 { format!("{}...", &s[..27]) } else { s.clone() }
                        }
                        _ => serde_json::to_string(val).unwrap_or_else(|_| "?".to_string()),
                    };
                    let val_truncated = if val_str.len() > max_width.saturating_sub(key.len() + 4) {
                        format!("{}...", &val_str[..max_width.saturating_sub(key.len() + 7).max(3)])
                    } else {
                        val_str
                    };
                    lines.push(format!("{}: {}", key, val_truncated));
                }
                if obj.len() > 10 {
                    lines.push(format!("... ({} more fields)", obj.len() - 10));
                }
            }
        }
    }

    lines
}

fn draw_cv_tab(f: &mut Frame, app: &mut App, area: Rect) {
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

fn draw_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    // For Business Data tab, show source toggle on right side
    if app.current_tab == Tab::BusinessData {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(35)])
            .split(area);

        let help = if app.bd_search_mode {
            "Type to filter | Enter: Apply | Esc: Cancel"
        } else if app.bd_tile_search_mode {
            "Type to search | ↑/↓: Navigate matches | Enter: Accept | Esc: Cancel"
        } else {
            match app.bd_view {
                BusinessDataView::GraphList => "↑/↓: Navigate | Enter: View | s: Toggle source",
                BusinessDataView::ResourceList => "↑/↓/jk: Navigate | Enter: View | /: Filter | s: Toggle source | Esc: Back",
                BusinessDataView::ResourceDetail => "↑/↓/jk: Navigate | →/l: Expand | ←/h: Collapse | /: Search | n/N: Next/prev | Esc: Back",
            }
        };

        let status = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
        f.render_widget(status, chunks[0]);

        // Source toggle indicator
        let source_label = format!("[s] Source: {}", app.bd_source.label());
        let source_style = match app.bd_source {
            BusinessDataSource::BusinessData => Style::default().fg(Color::Green),
            BusinessDataSource::Preindex => Style::default().fg(Color::Cyan),
        };
        let source_widget = Paragraph::new(source_label).style(source_style);
        f.render_widget(source_widget, chunks[1]);
    } else {
        let help = if app.search_mode {
            "Type to search | ↑/↓: Navigate matches | Enter: Accept | Esc: Cancel"
        } else {
            match app.current_tab {
                Tab::Graphs => match app.graphs_view {
                    GraphsView::List => "↑/↓: Navigate | Enter: Explore | Tab: Next tab | q: Quit",
                    GraphsView::Tree => "↑/↓/hjkl: Navigate | →/l: Expand | ←/h: Collapse | /: Search | n/N: Next/prev match | Esc: Back | q: Quit",
                },
                _ => "Tab: Next tab | q: Quit",
            }
        };

        let status = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
        f.render_widget(status, area);
    }
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

    // Keep cursor in the middle 50% of the view
    // Top margin = 25% of visible height, bottom margin = 25%
    let margin = visible_height / 4;
    let top_margin = margin.max(1);
    let bottom_margin = visible_height.saturating_sub(margin).max(1);

    let mut offset = current_offset;

    // Calculate where selection appears in current view
    let selection_in_view = selected.saturating_sub(offset);

    // If selection is in top margin zone, scroll up to center it more
    if selection_in_view < top_margin && offset > 0 {
        // Scroll so selection is at the top of center band
        offset = selected.saturating_sub(top_margin);
    }

    // If selection is in bottom margin zone, scroll down to center it more
    if selection_in_view >= bottom_margin {
        // Scroll so selection is at the bottom of center band
        offset = selected.saturating_sub(bottom_margin - 1);
    }

    // Hard limits: ensure selection is always visible
    if selected < offset {
        offset = selected;
    }
    if selected >= offset + visible_height {
        offset = selected - visible_height + 1;
    }

    // Don't scroll past the end
    offset.min(total_items.saturating_sub(visible_height))
}
