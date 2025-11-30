//! Application state and logic

use alizarin_core::{IndexedGraph, PrebuildInfo, PrebuildLoader, StaticNode};
use anyhow::Result;
use std::path::PathBuf;

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

/// A node in the tree view with expansion state
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub node_id: String,
    pub name: String,
    pub alias: String,
    pub datatype: String,
    pub depth: usize,
    pub expanded: bool,
    pub has_children: bool,
}

/// Application state
pub struct App {
    pub loader: PrebuildLoader,
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
}

impl App {
    pub fn new(prebuild_path: PathBuf) -> Result<Self> {
        let loader = PrebuildLoader::new(&prebuild_path)?;
        let info = loader.get_info()?;

        Ok(App {
            loader,
            info,
            current_tab: Tab::Graphs,
            graphs: Vec::new(),
            graphs_loaded: false,
            graph_list_selected: 0,
            graphs_view: GraphsView::List,
            tree_nodes: Vec::new(),
            tree_selected: 0,
            tree_scroll_offset: 0,
        })
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
        if self.current_tab == Tab::Graphs && self.graphs_view == GraphsView::List {
            self.enter_tree_view();
        }
    }

    pub fn on_escape(&mut self) {
        if self.current_tab == Tab::Graphs && self.graphs_view == GraphsView::Tree {
            self.graphs_view = GraphsView::List;
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
            Self::collect_tree_nodes(root, graph, 0, true)
        };

        self.tree_nodes = nodes;
    }

    /// Collect tree nodes recursively without borrowing self
    fn collect_tree_nodes(
        node: &StaticNode,
        graph: &IndexedGraph,
        depth: usize,
        expanded: bool,
    ) -> Vec<TreeNode> {
        let mut nodes = Vec::new();
        let has_children = graph.has_children(&node.nodeid);

        nodes.push(TreeNode {
            node_id: node.nodeid.clone(),
            name: node.name.clone(),
            alias: node.alias.clone().unwrap_or_default(),
            datatype: node.datatype.clone(),
            depth,
            expanded,
            has_children,
        });

        // If expanded, add children
        if expanded && has_children {
            let children = graph.get_children(&node.nodeid);
            let mut sorted_children: Vec<_> = children.into_iter().collect();
            sorted_children.sort_by(|a, b| {
                a.sortorder.unwrap_or(999).cmp(&b.sortorder.unwrap_or(999))
            });

            for child in sorted_children {
                // Children start collapsed
                nodes.extend(Self::collect_tree_nodes(child, graph, depth + 1, false));
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
            sorted_children.sort_by(|a, b| {
                a.sortorder.unwrap_or(999).cmp(&b.sortorder.unwrap_or(999))
            });

            sorted_children
                .iter()
                .map(|child| TreeNode {
                    node_id: child.nodeid.clone(),
                    name: child.name.clone(),
                    alias: child.alias.clone().unwrap_or_default(),
                    datatype: child.datatype.clone(),
                    depth: depth + 1,
                    expanded: false,
                    has_children: graph.has_children(&child.nodeid),
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
}
