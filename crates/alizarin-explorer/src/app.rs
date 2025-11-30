//! Application state and logic

use alizarin_core::{IndexedGraph, PrebuildInfo, PrebuildLoader, StaticNode, StaticNodegroup, StaticResourceSummary};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// The active tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Indexing,
    Graphs,
    BusinessData,
    ControlledVocabularies,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Indexing,
            Tab::Graphs,
            Tab::BusinessData,
            Tab::ControlledVocabularies,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Indexing => "Indexing",
            Tab::Graphs => "Graphs",
            Tab::BusinessData => "Business Data",
            Tab::ControlledVocabularies => "Controlled Vocabularies",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Indexing => 0,
            Tab::Graphs => 1,
            Tab::BusinessData => 2,
            Tab::ControlledVocabularies => 3,
        }
    }
}

/// View mode within the Graphs tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphsView {
    /// Table listing all graphs
    List,
    /// Tree view of a selected graph's structure
    Tree,
}

/// View mode within the Business Data tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessDataView {
    /// Graph selection list
    GraphList,
    /// Resource list for selected graph
    ResourceList,
}

/// Data source for business data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessDataSource {
    /// Load from business_data/**/*.json (full resources, extract summaries)
    BusinessData,
    /// Load from preindex/**/*.pi (pre-indexed summaries)
    Preindex,
}

impl BusinessDataSource {
    pub fn toggle(&self) -> Self {
        match self {
            BusinessDataSource::BusinessData => BusinessDataSource::Preindex,
            BusinessDataSource::Preindex => BusinessDataSource::BusinessData,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BusinessDataSource::BusinessData => "business_data/*.json",
            BusinessDataSource::Preindex => "preindex/*.pi",
        }
    }
}

/// Messages from background loader thread
pub enum LoaderMessage {
    /// A batch of resources was loaded
    ResourceBatch(Vec<StaticResourceSummary>),
    /// Total count is known
    TotalCount(usize),
    /// Counting progress (files counted so far, resources found so far)
    CountingProgress(usize, usize),
    /// Loading is complete
    Done,
    /// An error occurred
    Error(String),
}

/// A node in the tree view with expansion state
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub node_id: String,
    pub name: String,
    pub alias: String,
    pub datatype: String,
    pub nodegroup_id: Option<String>,
    pub depth: usize,
    pub expanded: bool,
    pub has_children: bool,
    pub is_collector: bool,
    pub is_required: bool,
    pub ontologyclass: Option<String>,
    pub description: Option<String>,
    /// Whether this node matches the current search (for highlighting)
    pub matches_search: bool,
    /// Whether this node is visible during search (matches or is ancestor of match)
    pub visible_in_search: bool,
}

/// Application state
pub struct App {
    pub loader: Arc<PrebuildLoader>,
    pub info: PrebuildInfo,
    pub current_tab: Tab,

    // Graphs tab state
    pub graphs: Vec<IndexedGraph>,
    pub graphs_loaded: bool,
    pub graph_list_selected: usize,
    pub graphs_view: GraphsView,

    // Tree view state
    pub tree_nodes: Vec<TreeNode>,
    pub tree_selected: usize,
    pub tree_scroll_offset: usize,

    // Search state
    pub search_mode: bool,
    pub search_query: String,
    pub search_error: Option<String>,
    pub search_case_sensitive: bool,
    /// Stores the tree state before search (node_id -> expanded)
    pre_search_expanded: std::collections::HashMap<String, bool>,
    /// Stores which node was selected before search
    pre_search_selected: usize,

    // Business Data tab state
    pub bd_view: BusinessDataView,
    pub bd_source: BusinessDataSource,
    pub bd_graph_selected: usize,
    pub bd_resources: Vec<StaticResourceSummary>,
    pub bd_resources_filtered: Vec<usize>, // indices into bd_resources that match filter
    pub bd_resource_selected: usize,
    pub bd_search_query: String,
    pub bd_search_mode: bool,
    pub bd_search_case_sensitive: bool,
    pub bd_loading: bool,
    pub bd_loaded_count: usize,
    pub bd_total_count: Option<usize>,
    pub bd_counting_files: usize,    // files counted so far
    pub bd_counting_resources: usize, // resources found during counting
    pub bd_has_more: bool,

    // Background loader
    bd_loader_rx: Option<Receiver<LoaderMessage>>,
    #[allow(dead_code)]
    bd_loader_handle: Option<JoinHandle<()>>,
}

impl App {
    pub fn new(prebuild_path: PathBuf) -> Result<Self> {
        let loader = PrebuildLoader::new(&prebuild_path)?;
        let info = loader.get_info()?;

        let mut app = App {
            loader: Arc::new(loader),
            info,
            current_tab: Tab::Graphs,
            graphs: Vec::new(),
            graphs_loaded: false,
            graph_list_selected: 0,
            graphs_view: GraphsView::List,
            tree_nodes: Vec::new(),
            tree_selected: 0,
            tree_scroll_offset: 0,
            search_mode: false,
            search_query: String::new(),
            search_error: None,
            search_case_sensitive: false, // case-insensitive by default
            pre_search_expanded: std::collections::HashMap::new(),
            pre_search_selected: 0,
            // Business Data
            bd_view: BusinessDataView::GraphList,
            bd_source: BusinessDataSource::BusinessData,
            bd_graph_selected: 0,
            bd_resources: Vec::new(),
            bd_resources_filtered: Vec::new(),
            bd_resource_selected: 0,
            bd_search_query: String::new(),
            bd_search_mode: false,
            bd_search_case_sensitive: false, // case-insensitive by default
            bd_loading: false,
            bd_loaded_count: 0,
            bd_total_count: None,
            bd_counting_files: 0,
            bd_counting_resources: 0,
            bd_has_more: false,
            bd_loader_rx: None,
            bd_loader_handle: None,
        };

        // Load graphs immediately since we start on the Graphs tab
        app.load_graphs();

        Ok(app)
    }

    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = self.current_tab.index();
        self.current_tab = tabs[(current_idx + 1) % tabs.len()];
        self.on_tab_changed();
    }

    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = self.current_tab.index();
        self.current_tab = tabs[(current_idx + tabs.len() - 1) % tabs.len()];
        self.on_tab_changed();
    }

    fn on_tab_changed(&mut self) {
        if self.current_tab == Tab::Graphs && !self.graphs_loaded {
            self.load_graphs();
        }
    }

    fn load_graphs(&mut self) {
        match self.loader.load_all_indexed_graphs() {
            Ok(graphs) => {
                self.graphs = graphs;
                self.graphs_loaded = true;
            }
            Err(e) => {
                eprintln!("Failed to load graphs: {}", e);
            }
        }
    }

    pub fn on_down(&mut self) {
        match self.current_tab {
            Tab::Graphs => match self.graphs_view {
                GraphsView::List => {
                    if !self.graphs.is_empty() {
                        self.graph_list_selected =
                            (self.graph_list_selected + 1).min(self.graphs.len() - 1);
                    }
                }
                GraphsView::Tree => {
                    if !self.tree_nodes.is_empty() {
                        self.tree_selected = (self.tree_selected + 1).min(self.tree_nodes.len() - 1);
                    }
                }
            },
            Tab::BusinessData => match self.bd_view {
                BusinessDataView::GraphList => {
                    if !self.graphs.is_empty() {
                        self.bd_graph_selected =
                            (self.bd_graph_selected + 1).min(self.graphs.len() - 1);
                    }
                }
                BusinessDataView::ResourceList => {
                    let max_idx = if self.bd_resources_filtered.is_empty() && self.bd_search_query.is_empty() {
                        self.bd_resources.len().saturating_sub(1)
                    } else {
                        self.bd_resources_filtered.len().saturating_sub(1)
                    };
                    self.bd_resource_selected = (self.bd_resource_selected + 1).min(max_idx);
                }
            },
            _ => {}
        }
    }

    pub fn on_up(&mut self) {
        match self.current_tab {
            Tab::Graphs => match self.graphs_view {
                GraphsView::List => {
                    if self.graph_list_selected > 0 {
                        self.graph_list_selected -= 1;
                    }
                }
                GraphsView::Tree => {
                    if self.tree_selected > 0 {
                        self.tree_selected -= 1;
                    }
                }
            },
            Tab::BusinessData => match self.bd_view {
                BusinessDataView::GraphList => {
                    if self.bd_graph_selected > 0 {
                        self.bd_graph_selected -= 1;
                    }
                }
                BusinessDataView::ResourceList => {
                    if self.bd_resource_selected > 0 {
                        self.bd_resource_selected -= 1;
                    }
                }
            },
            _ => {}
        }
    }

    pub fn on_right(&mut self) {
        if self.current_tab == Tab::Graphs && self.graphs_view == GraphsView::Tree {
            self.expand_selected_node();
        }
    }

    pub fn on_left(&mut self) {
        if self.current_tab == Tab::Graphs && self.graphs_view == GraphsView::Tree {
            self.collapse_selected_node();
        }
    }

    pub fn on_enter(&mut self) {
        match self.current_tab {
            Tab::Graphs => {
                if self.graphs_view == GraphsView::List {
                    self.enter_tree_view();
                }
            }
            Tab::BusinessData => {
                if self.bd_view == BusinessDataView::GraphList {
                    self.bd_enter_resource_list();
                }
            }
            _ => {}
        }
    }

    pub fn on_escape(&mut self) {
        match self.current_tab {
            Tab::Graphs => {
                if self.graphs_view == GraphsView::Tree {
                    self.graphs_view = GraphsView::List;
                }
            }
            Tab::BusinessData => {
                if self.bd_search_mode {
                    self.bd_exit_search_mode();
                } else if self.bd_view == BusinessDataView::ResourceList {
                    self.bd_view = BusinessDataView::GraphList;
                }
            }
            _ => {}
        }
    }

    fn enter_tree_view(&mut self) {
        if self.graph_list_selected >= self.graphs.len() {
            return;
        }

        self.graphs_view = GraphsView::Tree;
        self.tree_selected = 0;
        self.tree_scroll_offset = 0;
        self.rebuild_tree();
    }

    fn rebuild_tree(&mut self) {
        self.tree_nodes.clear();

        if self.graph_list_selected >= self.graphs.len() {
            return;
        }

        // Build tree nodes without recursive self-borrow
        let nodes = {
            let graph = &self.graphs[self.graph_list_selected];
            let root = graph.get_root();
            Self::collect_tree_nodes(root, graph, 0, true, false)
        };

        self.tree_nodes = nodes;
    }

    /// Rebuild tree with all nodes expanded (for search)
    fn rebuild_tree_fully_expanded(&mut self) {
        self.tree_nodes.clear();

        if self.graph_list_selected >= self.graphs.len() {
            return;
        }

        // Build tree nodes with all expanded
        let nodes = {
            let graph = &self.graphs[self.graph_list_selected];
            let root = graph.get_root();
            Self::collect_tree_nodes(root, graph, 0, true, true)
        };

        self.tree_nodes = nodes;
    }

    /// Collect tree nodes recursively without borrowing self
    /// If `expand_all` is true, all nodes will be expanded (for search mode)
    fn collect_tree_nodes(
        node: &StaticNode,
        graph: &IndexedGraph,
        depth: usize,
        expanded: bool,
        expand_all: bool,
    ) -> Vec<TreeNode> {
        let mut nodes = Vec::new();
        let has_children = graph.has_children(&node.nodeid);

        nodes.push(TreeNode {
            node_id: node.nodeid.clone(),
            name: node.name.clone(),
            alias: node.alias.clone().unwrap_or_default(),
            datatype: node.datatype.clone(),
            nodegroup_id: node.nodegroup_id.clone(),
            depth,
            expanded: expanded || expand_all,
            has_children,
            is_collector: node.is_collector,
            is_required: node.isrequired,
            ontologyclass: node.ontologyclass.clone(),
            description: node.description.as_ref().map(|d| d.to_string_default()),
            matches_search: false,
            visible_in_search: true,
        });

        // If expanded (or expand_all), add children
        if (expanded || expand_all) && has_children {
            let children = graph.get_children(&node.nodeid);
            let mut sorted_children: Vec<_> = children.into_iter().collect();
            // Sort alphabetically by alias (fallback to name if no alias)
            sorted_children.sort_by(|a, b| {
                let a_key = a.alias.as_deref().filter(|s| !s.is_empty()).unwrap_or(&a.name);
                let b_key = b.alias.as_deref().filter(|s| !s.is_empty()).unwrap_or(&b.name);
                a_key.to_lowercase().cmp(&b_key.to_lowercase())
            });

            for child in sorted_children {
                // If expand_all, expand children too; otherwise children start collapsed
                nodes.extend(Self::collect_tree_nodes(child, graph, depth + 1, expand_all, expand_all));
            }
        }

        nodes
    }

    fn expand_selected_node(&mut self) {
        if self.tree_selected >= self.tree_nodes.len() {
            return;
        }

        // Check if expandable without holding borrow
        let (should_expand, node_id, depth) = {
            let node = &self.tree_nodes[self.tree_selected];
            if !node.has_children || node.expanded {
                return;
            }
            (true, node.node_id.clone(), node.depth)
        };

        if !should_expand {
            return;
        }

        // Collect new nodes to insert
        let new_nodes: Vec<TreeNode> = {
            let graph = &self.graphs[self.graph_list_selected];

            if graph.get_node(&node_id).is_none() {
                return;
            }

            let children = graph.get_children(&node_id);
            let mut sorted_children: Vec<_> = children.into_iter().collect();
            // Sort alphabetically by alias (fallback to name if no alias)
            sorted_children.sort_by(|a, b| {
                let a_key = a.alias.as_deref().filter(|s| !s.is_empty()).unwrap_or(&a.name);
                let b_key = b.alias.as_deref().filter(|s| !s.is_empty()).unwrap_or(&b.name);
                a_key.to_lowercase().cmp(&b_key.to_lowercase())
            });

            sorted_children
                .iter()
                .map(|child| TreeNode {
                    node_id: child.nodeid.clone(),
                    name: child.name.clone(),
                    alias: child.alias.clone().unwrap_or_default(),
                    datatype: child.datatype.clone(),
                    nodegroup_id: child.nodegroup_id.clone(),
                    depth: depth + 1,
                    expanded: false,
                    has_children: graph.has_children(&child.nodeid),
                    is_collector: child.is_collector,
                    is_required: child.isrequired,
                    ontologyclass: child.ontologyclass.clone(),
                    description: child.description.as_ref().map(|d| d.to_string_default()),
                    matches_search: false,
                    visible_in_search: true,
                })
                .collect()
        };

        // Mark current node as expanded
        self.tree_nodes[self.tree_selected].expanded = true;

        // Insert children after current node
        let insert_pos = self.tree_selected + 1;
        for (i, node) in new_nodes.into_iter().enumerate() {
            self.tree_nodes.insert(insert_pos + i, node);
        }
    }

    fn collapse_selected_node(&mut self) {
        if self.tree_selected >= self.tree_nodes.len() {
            return;
        }

        let node = &self.tree_nodes[self.tree_selected];
        if !node.expanded {
            // If not expanded, go to parent (find node with lower depth)
            if node.depth > 0 {
                for i in (0..self.tree_selected).rev() {
                    if self.tree_nodes[i].depth < node.depth {
                        self.tree_selected = i;
                        break;
                    }
                }
            }
            return;
        }

        // Collapse: remove all children (nodes with greater depth until we hit same/lower depth)
        let current_depth = self.tree_nodes[self.tree_selected].depth;
        self.tree_nodes[self.tree_selected].expanded = false;

        // Remove children
        let mut remove_count = 0;
        for i in (self.tree_selected + 1)..self.tree_nodes.len() {
            if self.tree_nodes[i].depth > current_depth {
                remove_count += 1;
            } else {
                break;
            }
        }

        for _ in 0..remove_count {
            self.tree_nodes.remove(self.tree_selected + 1);
        }
    }

    pub fn selected_graph(&self) -> Option<&IndexedGraph> {
        self.graphs.get(self.graph_list_selected)
    }

    pub fn selected_tree_node(&self) -> Option<&TreeNode> {
        self.tree_nodes.get(self.tree_selected)
    }

    /// Get the nodegroup for the currently selected tree node
    pub fn selected_nodegroup(&self) -> Option<&StaticNodegroup> {
        let tree_node = self.selected_tree_node()?;
        let nodegroup_id = tree_node.nodegroup_id.as_ref()?;
        let graph = self.selected_graph()?;
        graph.nodegroups_by_id.get(nodegroup_id)
    }

    /// Get all unique nodegroup IDs currently visible in the tree
    pub fn visible_nodegroup_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .tree_nodes
            .iter()
            .filter_map(|n| n.nodegroup_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Enter search mode - save current tree state and expand all nodes
    pub fn enter_search_mode(&mut self) {
        if self.search_mode {
            return;
        }
        self.search_mode = true;
        self.search_query.clear();
        self.search_error = None;
        self.pre_search_selected = self.tree_selected;

        // Save expanded state for all nodes before expanding everything
        self.pre_search_expanded.clear();
        for node in &self.tree_nodes {
            self.pre_search_expanded
                .insert(node.node_id.clone(), node.expanded);
        }

        // Rebuild tree with all nodes expanded so we can search everything
        self.rebuild_tree_fully_expanded();

        // Reset search state on all nodes
        for node in &mut self.tree_nodes {
            node.matches_search = false;
            node.visible_in_search = true;
        }
    }

    /// Exit search mode - restore previous tree state
    pub fn exit_search_mode(&mut self) {
        if !self.search_mode {
            return;
        }
        self.search_mode = false;
        self.search_query.clear();
        self.search_error = None;

        // Rebuild tree to restore proper structure (this clears all nodes and rebuilds)
        self.rebuild_tree();

        // Reset search flags on rebuilt tree
        for node in &mut self.tree_nodes {
            node.matches_search = false;
            node.visible_in_search = true;
        }

        // Try to restore selection to same node if possible
        if let Some(selected_node_id) = self.pre_search_expanded.keys().nth(self.pre_search_selected) {
            if let Some(pos) = self.tree_nodes.iter().position(|n| &n.node_id == selected_node_id) {
                self.tree_selected = pos;
            }
        }
        self.tree_selected = self.tree_selected.min(self.tree_nodes.len().saturating_sub(1));
    }

    /// Handle character input during search
    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.apply_search_filter();
    }

    /// Handle backspace during search
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.apply_search_filter();
    }

    /// Toggle case sensitivity for graph tree search
    pub fn search_toggle_case(&mut self) {
        self.search_case_sensitive = !self.search_case_sensitive;
        self.apply_search_filter();
    }

    /// Apply search filter to tree nodes
    pub fn apply_search_filter(&mut self) {
        // Reset all nodes
        for node in &mut self.tree_nodes {
            node.matches_search = false;
            node.visible_in_search = false;
        }

        if self.search_query.is_empty() {
            // No search - show all nodes
            for node in &mut self.tree_nodes {
                node.visible_in_search = true;
            }
            self.search_error = None;
            return;
        }

        // Try to compile regex (case-insensitive by default)
        let regex = match regex::RegexBuilder::new(&self.search_query)
            .case_insensitive(!self.search_case_sensitive)
            .build()
        {
            Ok(r) => {
                self.search_error = None;
                r
            }
            Err(e) => {
                self.search_error = Some(format!("Invalid regex: {}", e));
                // Show all nodes on error
                for node in &mut self.tree_nodes {
                    node.visible_in_search = true;
                }
                return;
            }
        };

        // Find matching nodes - search in alias, name, and datatype
        let mut matching_indices: Vec<usize> = Vec::new();
        for (i, node) in self.tree_nodes.iter().enumerate() {
            let search_text = format!("{} {} {}", node.alias, node.name, node.datatype);
            if regex.is_match(&search_text) {
                matching_indices.push(i);
            }
        }

        // Mark matching nodes
        for &i in &matching_indices {
            self.tree_nodes[i].matches_search = true;
            self.tree_nodes[i].visible_in_search = true;
        }

        // Mark ancestors of matching nodes as visible
        // We need to walk up the tree for each match
        for &match_idx in &matching_indices {
            let match_depth = self.tree_nodes[match_idx].depth;
            let mut current_depth = match_depth;

            // Walk backwards to find ancestors
            for i in (0..match_idx).rev() {
                let node_depth = self.tree_nodes[i].depth;
                if node_depth < current_depth {
                    self.tree_nodes[i].visible_in_search = true;
                    current_depth = node_depth;
                    if node_depth == 0 {
                        break;
                    }
                }
            }
        }

        // Select first matching node if any
        if let Some(&first_match) = matching_indices.first() {
            self.tree_selected = first_match;
        }
    }

    /// Move to next search match
    pub fn search_next(&mut self) {
        if !self.search_mode {
            return;
        }
        let current = self.tree_selected;
        for i in (current + 1)..self.tree_nodes.len() {
            if self.tree_nodes[i].matches_search {
                self.tree_selected = i;
                return;
            }
        }
        // Wrap around
        for i in 0..current {
            if self.tree_nodes[i].matches_search {
                self.tree_selected = i;
                return;
            }
        }
    }

    /// Move to previous search match
    pub fn search_prev(&mut self) {
        if !self.search_mode {
            return;
        }
        let current = self.tree_selected;
        for i in (0..current).rev() {
            if self.tree_nodes[i].matches_search {
                self.tree_selected = i;
                return;
            }
        }
        // Wrap around
        for i in (current + 1..self.tree_nodes.len()).rev() {
            if self.tree_nodes[i].matches_search {
                self.tree_selected = i;
                return;
            }
        }
    }

    /// Get count of search matches
    pub fn search_match_count(&self) -> usize {
        self.tree_nodes.iter().filter(|n| n.matches_search).count()
    }

    // =========================================================================
    // Business Data Methods
    // =========================================================================

    /// Enter the resource list view for the selected graph
    fn bd_enter_resource_list(&mut self) {
        if self.bd_graph_selected >= self.graphs.len() {
            return;
        }

        self.bd_view = BusinessDataView::ResourceList;
        self.bd_resources.clear();
        self.bd_resources_filtered.clear();
        self.bd_resource_selected = 0;
        self.bd_search_query.clear();
        self.bd_loading = true;
        self.bd_loaded_count = 0;
        self.bd_total_count = None;
        self.bd_counting_files = 0;
        self.bd_counting_resources = 0;
        self.bd_has_more = true;

        // Spawn background loader thread
        let graph_id = self.graphs[self.bd_graph_selected].graph.graphid.clone();
        self.bd_start_background_load(graph_id);
    }

    /// Start background loading of resources
    fn bd_start_background_load(&mut self, graph_id: String) {
        let (tx, rx) = mpsc::channel();
        self.bd_loader_rx = Some(rx);

        let loader = Arc::clone(&self.loader);
        let source = self.bd_source;

        let handle = thread::spawn(move || {
            match source {
                BusinessDataSource::BusinessData => {
                    // Get files first
                    let files = match loader.find_business_data_files(&graph_id) {
                        Ok(f) => f,
                        Err(e) => {
                            let _ = tx.send(LoaderMessage::Error(e.to_string()));
                            let _ = tx.send(LoaderMessage::Done);
                            return;
                        }
                    };

                    // Fast count pass with progress updates
                    let mut file_counts = Vec::with_capacity(files.len());
                    let mut running_total = 0usize;
                    for (i, file) in files.iter().enumerate() {
                        match loader.fast_count_resources_in_file(file, &graph_id) {
                            Ok(count) => {
                                if count > 0 {
                                    file_counts.push((file.clone(), count));
                                    running_total += count;
                                }
                                let _ = tx.send(LoaderMessage::CountingProgress(i + 1, running_total));
                            }
                            Err(_) => continue,
                        }
                    }

                    // Send total count
                    let _ = tx.send(LoaderMessage::TotalCount(running_total));

                    // Load each file fully and send resources
                    for (file_path, _count) in file_counts {
                        match loader.load_resource_summaries_from_file(&file_path, &graph_id) {
                            Ok(summaries) => {
                                if !summaries.is_empty() {
                                    if tx.send(LoaderMessage::ResourceBatch(summaries)).is_err() {
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(LoaderMessage::Error(e.to_string()));
                            }
                        }
                    }
                }
                BusinessDataSource::Preindex => {
                    // Preindex: use existing batch approach
                    if let Ok(count) = loader.count_preindex_resources_for_graph(&graph_id) {
                        let _ = tx.send(LoaderMessage::TotalCount(count));
                    }

                    let batch_size = 50;
                    let mut offset = 0;
                    loop {
                        match loader.load_preindex_summaries(&graph_id, offset, batch_size) {
                            Ok((summaries, has_more)) => {
                                let batch_len = summaries.len();
                                if batch_len > 0 {
                                    if tx.send(LoaderMessage::ResourceBatch(summaries)).is_err() {
                                        break;
                                    }
                                }
                                offset += batch_len;
                                if !has_more || batch_len == 0 {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(LoaderMessage::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                }
            }

            let _ = tx.send(LoaderMessage::Done);
        });

        self.bd_loader_handle = Some(handle);
    }

    /// Poll the background loader for new data (call from main loop)
    pub fn bd_poll_loader(&mut self) {
        // Collect messages first to avoid borrow issues
        let messages: Vec<LoaderMessage> = {
            let rx = match &self.bd_loader_rx {
                Some(rx) => rx,
                None => return,
            };

            let mut msgs = Vec::new();
            loop {
                match rx.try_recv() {
                    Ok(msg) => msgs.push(msg),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        msgs.push(LoaderMessage::Done);
                        break;
                    }
                }
            }
            msgs
        };

        // Process collected messages
        let mut needs_filter = false;
        let mut is_done = false;

        for msg in messages {
            match msg {
                LoaderMessage::ResourceBatch(summaries) => {
                    self.bd_resources.extend(summaries);
                    self.bd_loaded_count = self.bd_resources.len();
                    needs_filter = true;
                }
                LoaderMessage::TotalCount(count) => {
                    self.bd_total_count = Some(count);
                }
                LoaderMessage::CountingProgress(files, resources) => {
                    self.bd_counting_files = files;
                    self.bd_counting_resources = resources;
                }
                LoaderMessage::Done => {
                    self.bd_loading = false;
                    self.bd_has_more = false;
                    is_done = true;
                }
                LoaderMessage::Error(e) => {
                    eprintln!("Loader error: {}", e);
                    self.bd_loading = false;
                    self.bd_has_more = false;
                    is_done = true;
                }
            }
        }

        // Apply filter after processing all messages
        if needs_filter && !self.bd_search_query.is_empty() {
            self.bd_apply_search_filter();
        }

        if is_done {
            self.bd_loader_rx = None;
        }
    }

    /// Enter search mode for business data
    pub fn bd_enter_search_mode(&mut self) {
        self.bd_search_mode = true;
        self.bd_search_query.clear();
    }

    /// Exit search mode for business data
    pub fn bd_exit_search_mode(&mut self) {
        self.bd_search_mode = false;
        self.bd_search_query.clear();
        self.bd_resources_filtered.clear();
        self.bd_resource_selected = 0;
    }

    /// Handle character input during business data search
    pub fn bd_search_input(&mut self, c: char) {
        self.bd_search_query.push(c);
        self.bd_apply_search_filter();
    }

    /// Handle backspace during business data search
    pub fn bd_search_backspace(&mut self) {
        self.bd_search_query.pop();
        self.bd_apply_search_filter();
    }

    /// Toggle case sensitivity for business data search
    pub fn bd_search_toggle_case(&mut self) {
        self.bd_search_case_sensitive = !self.bd_search_case_sensitive;
        self.bd_apply_search_filter();
    }

    /// Apply search filter to resources
    pub fn bd_apply_search_filter(&mut self) {
        self.bd_resources_filtered.clear();

        if self.bd_search_query.is_empty() {
            self.bd_resource_selected = 0;
            return;
        }

        // Compile regex (case-insensitive by default)
        let regex = match regex::RegexBuilder::new(&self.bd_search_query)
            .case_insensitive(!self.bd_search_case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(_) => return,
        };

        // Filter resources by name
        for (i, resource) in self.bd_resources.iter().enumerate() {
            if regex.is_match(&resource.name) {
                self.bd_resources_filtered.push(i);
            }
        }

        self.bd_resource_selected = 0;
    }

    /// Get the currently selected resource
    pub fn bd_selected_resource(&self) -> Option<&StaticResourceSummary> {
        if self.bd_resources_filtered.is_empty() && self.bd_search_query.is_empty() {
            // No filter active - use direct index
            self.bd_resources.get(self.bd_resource_selected)
        } else if !self.bd_resources_filtered.is_empty() {
            // Filter active - use filtered index
            self.bd_resources_filtered
                .get(self.bd_resource_selected)
                .and_then(|&idx| self.bd_resources.get(idx))
        } else {
            // Filter active but no matches
            None
        }
    }

    /// Get the selected graph for business data
    pub fn bd_selected_graph(&self) -> Option<&IndexedGraph> {
        self.graphs.get(self.bd_graph_selected)
    }

    /// Get list of resources to display (filtered or all)
    pub fn bd_display_resources(&self) -> Vec<&StaticResourceSummary> {
        if self.bd_resources_filtered.is_empty() && self.bd_search_query.is_empty() {
            self.bd_resources.iter().collect()
        } else {
            self.bd_resources_filtered
                .iter()
                .filter_map(|&idx| self.bd_resources.get(idx))
                .collect()
        }
    }

    /// Get resource count for display
    pub fn bd_display_count(&self) -> usize {
        if self.bd_resources_filtered.is_empty() && self.bd_search_query.is_empty() {
            self.bd_resources.len()
        } else {
            self.bd_resources_filtered.len()
        }
    }

    /// Toggle the data source and reload if in resource list view
    pub fn bd_toggle_source(&mut self) {
        self.bd_source = self.bd_source.toggle();

        // If we're viewing resources, reload with new source
        if self.bd_view == BusinessDataView::ResourceList && self.bd_graph_selected < self.graphs.len() {
            self.bd_resources.clear();
            self.bd_resources_filtered.clear();
            self.bd_resource_selected = 0;
            self.bd_loading = true;
            self.bd_loaded_count = 0;
            self.bd_total_count = None;
            self.bd_has_more = true;

            // Start background loading with new source
            let graph_id = self.graphs[self.bd_graph_selected].graph.graphid.clone();
            self.bd_start_background_load(graph_id);
        }
    }
}
