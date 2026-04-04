//! Card hierarchy index for UI-oriented traversal.
//!
//! Builds a pre-computed index of the card tree from graph metadata,
//! enabling card-based traversal as an alternative to node/edge traversal.

use std::collections::HashMap;

use super::cards::{StaticCard, StaticCardsXNodesXWidgets};
use super::nodes::StaticNodegroup;
use super::translatable::StaticTranslatableString;
use super::StaticNode;

/// A reference to a widget entry within a card, with resolved metadata.
#[derive(Debug, Clone)]
pub struct CardWidgetRef {
    pub node_id: String,
    pub node_alias: String,
    pub widget_id: String,
    /// Resolved widget name (e.g. "text-widget"), or empty if unknown
    pub widget_name: String,
    pub label: StaticTranslatableString,
    pub config: serde_json::Value,
    pub sortorder: i32,
    pub visible: bool,
}

/// Pre-computed index of the card hierarchy for a graph.
///
/// Built once at graph load time. Enables card-based traversal that
/// reuses the existing pseudo_cache (no data duplication).
#[derive(Debug, Clone)]
pub struct CardIndex {
    /// card_id -> child card_ids, sorted by child card sortorder
    pub card_children: HashMap<String, Vec<String>>,
    /// Root cards (nodegroup has no parentnodegroup_id), sorted by sortorder
    pub root_card_ids: Vec<String>,
    /// nodegroup_id -> card_id
    pub card_by_nodegroup: HashMap<String, String>,
    /// card_id -> widget entries, sorted by sortorder
    pub widgets_by_card: HashMap<String, Vec<CardWidgetRef>>,
    /// node_id -> node alias
    pub alias_by_node_id: HashMap<String, String>,
    /// card_id -> StaticCard reference data (name, component_id, etc.)
    pub cards_by_id: HashMap<String, CardRef>,
}

/// Lightweight card reference for traversal output.
#[derive(Debug, Clone)]
pub struct CardRef {
    pub cardid: String,
    pub name: StaticTranslatableString,
    pub component_id: String,
    pub nodegroup_id: String,
    pub sortorder: Option<i32>,
    pub visible: bool,
    pub active: bool,
}

impl From<&StaticCard> for CardRef {
    fn from(card: &StaticCard) -> Self {
        CardRef {
            cardid: card.cardid.clone(),
            name: card.name.clone(),
            component_id: card.component_id.clone(),
            nodegroup_id: card.nodegroup_id.clone(),
            sortorder: card.sortorder,
            visible: card.visible,
            active: card.active,
        }
    }
}

impl CardIndex {
    /// Collect nodegroup IDs needed to serialize a card (and its descendants).
    ///
    /// `max_depth`: `None` = all descendants, `Some(0)` = this card only,
    /// `Some(1)` = this card + immediate children, etc.
    pub fn nodegroup_ids_for_card(&self, card_id: &str, max_depth: Option<usize>) -> Vec<String> {
        let mut result = Vec::new();
        self.collect_nodegroups(card_id, max_depth, &mut result);
        result
    }

    /// Collect nodegroup IDs for all root cards (and their descendants).
    pub fn nodegroup_ids_for_roots(&self, max_depth: Option<usize>) -> Vec<String> {
        let mut result = Vec::new();
        for card_id in &self.root_card_ids {
            self.collect_nodegroups(card_id, max_depth, &mut result);
        }
        result
    }

    fn collect_nodegroups(&self, card_id: &str, max_depth: Option<usize>, out: &mut Vec<String>) {
        if let Some(card) = self.cards_by_id.get(card_id) {
            if !out.contains(&card.nodegroup_id) {
                out.push(card.nodegroup_id.clone());
            }

            if max_depth == Some(0) {
                return;
            }

            let child_depth = max_depth.map(|d| d.saturating_sub(1));
            if let Some(children) = self.card_children.get(card_id) {
                for child_id in children {
                    self.collect_nodegroups(child_id, child_depth, out);
                }
            }
        }
    }
}

/// Build a CardIndex from graph components.
///
/// The `widget_name_resolver` function maps widget_id -> widget_name.
/// Pass `crate::graph_mutator::get_widget_name_by_id` for the default resolver.
pub fn build_card_index(
    cards: &[StaticCard],
    cxnxws: &[StaticCardsXNodesXWidgets],
    nodegroups: &[StaticNodegroup],
    nodes: &[StaticNode],
    widget_name_resolver: impl Fn(&str) -> Option<String>,
) -> CardIndex {
    // 1. Build alias_by_node_id
    let alias_by_node_id: HashMap<String, String> = nodes
        .iter()
        .filter_map(|n| {
            n.alias
                .as_ref()
                .filter(|a| !a.is_empty())
                .map(|a| (n.nodeid.clone(), a.clone()))
        })
        .collect();

    // 2. Build card_by_nodegroup and cards_by_id
    let mut card_by_nodegroup: HashMap<String, String> = HashMap::new();
    let mut cards_by_id: HashMap<String, CardRef> = HashMap::new();
    for card in cards {
        card_by_nodegroup.insert(card.nodegroup_id.clone(), card.cardid.clone());
        cards_by_id.insert(card.cardid.clone(), CardRef::from(card));
    }

    // 3. Build nodegroup lookup
    let ng_by_id: HashMap<&str, &StaticNodegroup> = nodegroups
        .iter()
        .map(|ng| (ng.nodegroupid.as_str(), ng))
        .collect();

    // 4. Build card_children and root_card_ids
    let mut card_children: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_card_ids: Vec<String> = Vec::new();

    for card in cards {
        let ng = ng_by_id.get(card.nodegroup_id.as_str());
        let parent_ng_id = ng.and_then(|ng| ng.parentnodegroup_id.as_ref());

        match parent_ng_id {
            Some(parent_ng) => {
                if let Some(parent_card_id) = card_by_nodegroup.get(parent_ng) {
                    card_children
                        .entry(parent_card_id.clone())
                        .or_default()
                        .push(card.cardid.clone());
                } else {
                    // Parent nodegroup has no card — treat as root
                    root_card_ids.push(card.cardid.clone());
                }
            }
            None => {
                root_card_ids.push(card.cardid.clone());
            }
        }
    }

    // Sort children by sortorder
    let card_sortorder = |card_id: &str| -> i32 {
        cards_by_id
            .get(card_id)
            .and_then(|c| c.sortorder)
            .unwrap_or(0)
    };
    for children in card_children.values_mut() {
        children.sort_by_key(|id| card_sortorder(id));
    }
    root_card_ids.sort_by_key(|id| card_sortorder(id));

    // 5. Build widgets_by_card
    let mut widgets_by_card: HashMap<String, Vec<CardWidgetRef>> = HashMap::new();
    for cxnxw in cxnxws {
        let node_alias = alias_by_node_id
            .get(&cxnxw.node_id)
            .cloned()
            .unwrap_or_default();
        let widget_name = widget_name_resolver(&cxnxw.widget_id).unwrap_or_default();

        let widget_ref = CardWidgetRef {
            node_id: cxnxw.node_id.clone(),
            node_alias,
            widget_id: cxnxw.widget_id.clone(),
            widget_name,
            label: cxnxw.label.clone(),
            config: cxnxw.config.clone(),
            sortorder: cxnxw.sortorder.unwrap_or(0),
            visible: cxnxw.visible,
        };

        widgets_by_card
            .entry(cxnxw.card_id.clone())
            .or_default()
            .push(widget_ref);
    }

    // Sort widgets by sortorder within each card
    for widgets in widgets_by_card.values_mut() {
        widgets.sort_by_key(|w| w.sortorder);
    }

    CardIndex {
        card_children,
        root_card_ids,
        card_by_nodegroup,
        widgets_by_card,
        alias_by_node_id,
        cards_by_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::translatable::StaticTranslatableString;

    fn make_node(id: &str, alias: &str, ng_id: &str) -> StaticNode {
        StaticNode {
            nodeid: id.to_string(),
            name: alias.to_string(),
            alias: Some(alias.to_string()),
            datatype: "string".to_string(),
            nodegroup_id: Some(ng_id.to_string()),
            graph_id: "test-graph".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: false,
            sortorder: Some(0),
            config: Default::default(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        }
    }

    fn make_card(id: &str, name: &str, ng_id: &str, sortorder: i32) -> StaticCard {
        StaticCard {
            cardid: id.to_string(),
            name: StaticTranslatableString::from_string(name),
            nodegroup_id: ng_id.to_string(),
            component_id: "default".to_string(),
            graph_id: "test-graph".to_string(),
            active: true,
            visible: true,
            sortorder: Some(sortorder),
            config: None,
            constraints: vec![],
            cssclass: None,
            description: None,
            helpenabled: false,
            helptext: StaticTranslatableString::empty(),
            helptitle: StaticTranslatableString::empty(),
            instructions: StaticTranslatableString::empty(),
            is_editable: Some(true),
            source_identifier_id: None,
        }
    }

    fn make_ng(id: &str, parent: Option<&str>) -> StaticNodegroup {
        StaticNodegroup {
            nodegroupid: id.to_string(),
            cardinality: Some("1".to_string()),
            parentnodegroup_id: parent.map(|s| s.to_string()),
            legacygroupid: None,
            grouping_node_id: None,
        }
    }

    fn make_cxnxw(
        card_id: &str,
        node_id: &str,
        widget_id: &str,
        sortorder: i32,
    ) -> StaticCardsXNodesXWidgets {
        StaticCardsXNodesXWidgets {
            card_id: card_id.to_string(),
            node_id: node_id.to_string(),
            widget_id: widget_id.to_string(),
            id: format!("cxnxw-{}-{}", card_id, node_id),
            label: StaticTranslatableString::from_string("Label"),
            config: serde_json::Value::Object(serde_json::Map::new()),
            sortorder: Some(sortorder),
            visible: true,
            source_identifier_id: None,
        }
    }

    #[test]
    fn test_build_card_index_basic() {
        let nodes = vec![
            make_node("n1", "field_a", "ng1"),
            make_node("n2", "field_b", "ng1"),
            make_node("n3", "field_c", "ng2"),
        ];
        let nodegroups = vec![make_ng("ng1", None), make_ng("ng2", Some("ng1"))];
        let cards = vec![
            make_card("card1", "Parent Card", "ng1", 0),
            make_card("card2", "Child Card", "ng2", 0),
        ];
        let cxnxws = vec![
            make_cxnxw("card1", "n1", "widget-text", 0),
            make_cxnxw("card1", "n2", "widget-concept", 1),
            make_cxnxw("card2", "n3", "widget-date", 0),
        ];

        let index = build_card_index(&cards, &cxnxws, &nodegroups, &nodes, |wid| {
            Some(format!("resolved-{}", wid))
        });

        // Root cards
        assert_eq!(index.root_card_ids, vec!["card1"]);

        // Card children
        assert_eq!(
            index.card_children.get("card1").unwrap(),
            &vec!["card2".to_string()]
        );
        assert!(index.card_children.get("card2").is_none());

        // Widgets
        let card1_widgets = index.widgets_by_card.get("card1").unwrap();
        assert_eq!(card1_widgets.len(), 2);
        assert_eq!(card1_widgets[0].node_alias, "field_a");
        assert_eq!(card1_widgets[1].node_alias, "field_b");

        let card2_widgets = index.widgets_by_card.get("card2").unwrap();
        assert_eq!(card2_widgets.len(), 1);
        assert_eq!(card2_widgets[0].node_alias, "field_c");

        // Alias reverse lookup
        assert_eq!(index.alias_by_node_id.get("n1").unwrap(), "field_a");
    }

    #[test]
    fn test_nodegroup_ids_for_card() {
        let nodes = vec![
            make_node("n1", "a", "ng1"),
            make_node("n2", "b", "ng2"),
            make_node("n3", "c", "ng3"),
        ];
        let nodegroups = vec![
            make_ng("ng1", None),
            make_ng("ng2", Some("ng1")),
            make_ng("ng3", Some("ng2")),
        ];
        let cards = vec![
            make_card("card1", "Root", "ng1", 0),
            make_card("card2", "Child", "ng2", 0),
            make_card("card3", "Grandchild", "ng3", 0),
        ];
        let index = build_card_index(&cards, &[], &nodegroups, &nodes, |_| None);

        // Unlimited depth: all nodegroups
        let all = index.nodegroup_ids_for_card("card1", None);
        assert_eq!(all, vec!["ng1", "ng2", "ng3"]);

        // Depth 0: just this card
        let just_root = index.nodegroup_ids_for_card("card1", Some(0));
        assert_eq!(just_root, vec!["ng1"]);

        // Depth 1: root + immediate children
        let one_level = index.nodegroup_ids_for_card("card1", Some(1));
        assert_eq!(one_level, vec!["ng1", "ng2"]);

        // From middle of tree
        let from_child = index.nodegroup_ids_for_card("card2", None);
        assert_eq!(from_child, vec!["ng2", "ng3"]);

        // All roots
        let roots = index.nodegroup_ids_for_roots(Some(0));
        assert_eq!(roots, vec!["ng1"]);
    }

    #[test]
    fn test_card_index_sortorder() {
        let nodes = vec![
            make_node("n1", "a", "ng1"),
            make_node("n2", "b", "ng2"),
            make_node("n3", "c", "ng3"),
        ];
        let nodegroups = vec![
            make_ng("ng1", None),
            make_ng("ng2", None),
            make_ng("ng3", None),
        ];
        let cards = vec![
            make_card("card1", "Third", "ng1", 2),
            make_card("card2", "First", "ng2", 0),
            make_card("card3", "Second", "ng3", 1),
        ];
        let cxnxws = vec![];

        let index = build_card_index(&cards, &cxnxws, &nodegroups, &nodes, |_| None);

        // Root cards should be sorted by sortorder
        assert_eq!(index.root_card_ids, vec!["card2", "card3", "card1"]);
    }
}
