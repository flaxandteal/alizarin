//! Arches resource instance lifecycles.
//!
//! A lifecycle is a small state graph attached to a resource model (Arches 8+).
//! Each resource instance holds at most one state id; Alizarin stores it
//! sparsely, so an absent state means the lifecycle's initial state.
//!
//! The graph keeps the exported lifecycle as raw JSON (exact round-trip); it is
//! compiled once into a [`LifecycleIndex`] on first use. Every check after that
//! is a short scan of a handful of ids plus a bit test.
//!
//! Permissions (`can_edit` / `can_delete`) are carried for round-trip and host
//! apps but never consulted here.

use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;

use super::resources::{StaticResource, StaticResourceMetadata};

/// One state of an exported Arches lifecycle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticLifecycleState {
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub action_label: serde_json::Value,
    #[serde(default)]
    pub can_delete_resource_instances: bool,
    #[serde(default)]
    pub can_edit_resource_instances: bool,
    pub id: String,
    #[serde(default)]
    pub is_initial_state: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub name: serde_json::Value,
    #[serde(default, deserialize_with = "ids_or_objects")]
    pub next_resource_instance_lifecycle_states: Vec<String>,
    #[serde(default, deserialize_with = "ids_or_objects")]
    pub previous_resource_instance_lifecycle_states: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_instance_lifecycle_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// An exported Arches lifecycle, as inlined into graph JSON by the Arches
/// graph exporter (`resource_instance_lifecycle`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticLifecycle {
    pub id: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub name: serde_json::Value,
    #[serde(default)]
    pub resource_instance_lifecycle_states: Vec<StaticLifecycleState>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Edge lists are ids in Arches graph exports, but a full model serialisation
/// nests the related states; accept either.
fn ids_or_objects<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    let items: Vec<serde_json::Value> = Option::deserialize(d)?.unwrap_or_default();
    items
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => Ok(s),
            serde_json::Value::Object(ref m) => m
                .get("id")
                .and_then(|id| id.as_str())
                .map(str::to_string)
                .ok_or_else(|| serde::de::Error::custom("lifecycle state edge has no id")),
            _ => Err(serde::de::Error::custom(
                "lifecycle state edge must be an id or a state object",
            )),
        })
        .collect()
}

/// Most states a compiled lifecycle supports (one bit each in a `u64`).
pub const MAX_LIFECYCLE_STATES: usize = 64;

/// A lifecycle compiled for checks: state ids by position, the initial
/// state's position, and a bitmask of permitted targets per state
/// (`next ∪ previous` edges, as the Arches UI offers both).
///
/// A graph with no lifecycle (branches, draft graphs, pre-Arches-8 exports)
/// compiles to a synthetic Perpetual: one state with no id and no edges.
#[derive(Clone, Debug)]
pub struct LifecycleIndex {
    lifecycle_id: Option<Arc<str>>,
    ids: Vec<Arc<str>>,
    initial: u8,
    permitted: Vec<u64>,
}

impl LifecycleIndex {
    /// The synthetic Perpetual lifecycle used when a graph defines none.
    pub fn perpetual() -> Self {
        Self {
            lifecycle_id: None,
            ids: Vec::new(),
            initial: 0,
            permitted: vec![0],
        }
    }

    /// Compile a graph's raw `resource_instance_lifecycle` value.
    pub fn from_value(value: Option<&serde_json::Value>) -> Result<Self, String> {
        match value {
            None | Some(serde_json::Value::Null) => Ok(Self::perpetual()),
            Some(v) => {
                let lifecycle = StaticLifecycle::deserialize(v)
                    .map_err(|e| format!("Invalid resource_instance_lifecycle: {e}"))?;
                Self::compile(&lifecycle)
            }
        }
    }

    /// Compile a typed lifecycle.
    pub fn compile(lifecycle: &StaticLifecycle) -> Result<Self, String> {
        let states = &lifecycle.resource_instance_lifecycle_states;
        if states.is_empty() {
            return Err(format!("Lifecycle '{}' has no states", lifecycle.id));
        }
        if states.len() > MAX_LIFECYCLE_STATES {
            return Err(format!(
                "Lifecycle '{}' has {} states; at most {MAX_LIFECYCLE_STATES} are supported",
                lifecycle.id,
                states.len()
            ));
        }
        let ids: Vec<Arc<str>> = states.iter().map(|s| Arc::from(s.id.as_str())).collect();
        let pos = |id: &str| ids.iter().position(|s| &**s == id);

        let mut initial = None;
        let mut permitted = vec![0u64; states.len()];
        for (i, state) in states.iter().enumerate() {
            if state.is_initial_state {
                if initial.is_some() {
                    return Err(format!(
                        "Lifecycle '{}' has more than one initial state",
                        lifecycle.id
                    ));
                }
                initial = Some(i as u8);
            }
            for target in state
                .next_resource_instance_lifecycle_states
                .iter()
                .chain(&state.previous_resource_instance_lifecycle_states)
            {
                let t = pos(target).ok_or_else(|| {
                    format!(
                        "Lifecycle '{}': state '{}' links to unknown state '{target}'",
                        lifecycle.id, state.id
                    )
                })?;
                permitted[i] |= 1 << t;
            }
        }
        let initial =
            initial.ok_or_else(|| format!("Lifecycle '{}' has no initial state", lifecycle.id))?;

        Ok(Self {
            lifecycle_id: Some(Arc::from(lifecycle.id.as_str())),
            ids,
            initial,
            permitted,
        })
    }

    /// The lifecycle id, or `None` for the synthetic Perpetual.
    pub fn lifecycle_id(&self) -> Option<&str> {
        self.lifecycle_id.as_deref()
    }

    /// The initial state's id, or `None` for the synthetic Perpetual.
    pub fn initial_state_id(&self) -> Option<&str> {
        self.ids.get(self.initial as usize).map(|s| &**s)
    }

    /// Position of a state id in this lifecycle.
    pub fn position(&self, state_id: &str) -> Option<u8> {
        self.ids
            .iter()
            .position(|s| &**s == state_id)
            .map(|i| i as u8)
    }

    /// Whether `to` is a permitted target from `from` (positions).
    pub fn is_permitted(&self, from: u8, to: u8) -> bool {
        self.permitted
            .get(from as usize)
            .is_some_and(|mask| mask & (1 << to) != 0)
    }

    /// Resolve a resource's stored state against this lifecycle.
    pub fn effective_state<'a>(&'a self, meta: &'a StaticResourceMetadata) -> EffectiveState<'a> {
        match meta.stored_state() {
            StoredState::Conflict(ids) => EffectiveState::Conflict(ids),
            StoredState::Initial => EffectiveState::State {
                position: self.initial,
                id: self.initial_state_id(),
                is_initial: true,
            },
            StoredState::Known(id) => match self.position(id) {
                Some(p) => EffectiveState::State {
                    position: p,
                    id: Some(&*self.ids[p as usize]),
                    is_initial: p == self.initial,
                },
                None => EffectiveState::Unknown(id),
            },
        }
    }
}

/// What a resource header stores, before resolving against a lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredState<'a> {
    /// No state stored: the lifecycle's initial state.
    Initial,
    /// An explicit state id.
    Known(&'a str),
    /// Composed layers disagreed; the distinct ids they gave.
    Conflict(&'a [String]),
}

/// A stored state resolved against a lifecycle. Never falls back silently:
/// an id the lifecycle doesn't define is `Unknown`, a disagreement is `Conflict`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectiveState<'a> {
    State {
        position: u8,
        /// `None` only for the synthetic Perpetual.
        id: Option<&'a str>,
        is_initial: bool,
    },
    Unknown(&'a str),
    Conflict(&'a [String]),
}

impl StaticResourceMetadata {
    /// The stored lifecycle state of this header.
    pub fn stored_state(&self) -> StoredState<'_> {
        if !self.lifecycle_state_conflict.is_empty() {
            StoredState::Conflict(&self.lifecycle_state_conflict)
        } else if let Some(ref id) = self.resource_instance_lifecycle_state_id {
            StoredState::Known(id)
        } else {
            StoredState::Initial
        }
    }
}

/// A state change, for the caller to log or persist.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionEvent {
    pub resourceinstanceid: String,
    /// The state left; `None` when leaving a Conflict.
    pub from: Option<String>,
    pub to: String,
    /// The ids in conflict, when the transition resolved one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolved_conflict: Vec<String>,
}

/// Move a resource to `target`, returning a tiles-free header overlay and the
/// event. Pure: nothing is persisted, and lifecycle handlers are the caller's.
///
/// The target must belong to the lifecycle. With `strict`, it must also be a
/// permitted target (`next ∪ previous`) of the current state, and nothing
/// leaves a `Conflict` or an unknown state. Non-strict matches Arches' server,
/// which checks only membership; ask for it explicitly at the call site.
pub fn transition(
    meta: &StaticResourceMetadata,
    lifecycle: &LifecycleIndex,
    target: &str,
    strict: bool,
) -> Result<(StaticResource, TransitionEvent), String> {
    let to = lifecycle
        .position(target)
        .ok_or_else(|| match lifecycle.lifecycle_id() {
            Some(lc) => format!("State '{target}' is not in lifecycle '{lc}'"),
            None => format!("Graph has no lifecycle; cannot move to '{target}'"),
        })?;

    let (from, resolved_conflict) = match lifecycle.effective_state(meta) {
        EffectiveState::State { position, id, .. } => {
            if strict && !lifecycle.is_permitted(position, to) {
                return Err(format!(
                    "Transition {} -> {target} is not permitted by the lifecycle",
                    id.unwrap_or("(none)")
                ));
            }
            (id.map(str::to_string), Vec::new())
        }
        EffectiveState::Unknown(id) => {
            if strict {
                return Err(format!(
                    "Resource '{}' is in state '{id}', which its lifecycle does not define",
                    meta.resourceinstanceid
                ));
            }
            (Some(id.to_string()), Vec::new())
        }
        EffectiveState::Conflict(ids) => {
            if strict {
                return Err(format!(
                    "Resource '{}' has conflicting lifecycle states {ids:?}; \
                     resolve with a non-strict transition",
                    meta.resourceinstanceid
                ));
            }
            (None, ids.to_vec())
        }
    };

    let mut header = meta.clone();
    header.resource_instance_lifecycle_state_id = Some(target.to_string());
    header.lifecycle_state_conflict.clear();

    let event = TransitionEvent {
        resourceinstanceid: meta.resourceinstanceid.clone(),
        from,
        to: target.to_string(),
        resolved_conflict,
    };
    let overlay = StaticResource {
        resourceinstance: header,
        tiles: None,
        metadata: Default::default(),
        cache: None,
        scopes: None,
        tiles_loaded: None,
    };
    Ok((overlay, event))
}

/// A resource's resolved lifecycle state, as the bindings return it. Every
/// field is always written, so the shape is the same on every backend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleStateReport {
    /// `"state"`, `"unknown"` (an id the lifecycle doesn't define) or `"conflict"`.
    pub status: String,
    /// The state id; `null` for a Conflict and for the synthetic Perpetual.
    pub state_id: Option<String>,
    pub is_initial: bool,
    /// The ids the layers disagreed on; empty unless `status` is `"conflict"`.
    pub conflict: Vec<String>,
}

impl From<EffectiveState<'_>> for LifecycleStateReport {
    fn from(state: EffectiveState<'_>) -> Self {
        let (status, state_id, is_initial, conflict) = match state {
            EffectiveState::State { id, is_initial, .. } => {
                ("state", id.map(str::to_string), is_initial, Vec::new())
            }
            EffectiveState::Unknown(id) => ("unknown", Some(id.to_string()), false, Vec::new()),
            EffectiveState::Conflict(ids) => ("conflict", None, false, ids.to_vec()),
        };
        Self {
            status: status.to_string(),
            state_id,
            is_initial,
            conflict,
        }
    }
}

/// The result of [`transition_from_json`]: the tiles-free overlay record and
/// the event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionResult {
    pub resource: StaticResource,
    pub event: TransitionEvent,
}

fn parse_resource_and_lifecycle(
    resource_json: &str,
    lifecycle_json: &str,
) -> Result<(StaticResource, LifecycleIndex), String> {
    let resource: StaticResource = serde_json::from_str(resource_json)
        .map_err(|e| format!("Failed to parse resource: {e}"))?;
    let lifecycle: serde_json::Value = serde_json::from_str(lifecycle_json)
        .map_err(|e| format!("Failed to parse lifecycle: {e}"))?;
    Ok((resource, LifecycleIndex::from_value(Some(&lifecycle))?))
}

/// JSON-boundary form of [`LifecycleIndex::effective_state`] for the bindings.
/// `lifecycle_json` is a graph's `resource_instance_lifecycle` (`"null"` for
/// none: the synthetic Perpetual). A lifecycle is a handful of states, so
/// compiling it per call is cheap; Rust callers working in bulk should use
/// [`StaticGraph::lifecycle`](super::StaticGraph::lifecycle), which caches it.
pub fn lifecycle_state_from_json(
    resource_json: &str,
    lifecycle_json: &str,
) -> Result<LifecycleStateReport, String> {
    let (resource, lifecycle) = parse_resource_and_lifecycle(resource_json, lifecycle_json)?;
    Ok(lifecycle.effective_state(&resource.resourceinstance).into())
}

/// JSON-boundary form of [`transition`] for the bindings; arguments as
/// [`lifecycle_state_from_json`]. `strict` is required here: each binding
/// defaults it to `true`.
pub fn transition_from_json(
    resource_json: &str,
    lifecycle_json: &str,
    target: &str,
    strict: bool,
) -> Result<TransitionResult, String> {
    let (resource, lifecycle) = parse_resource_and_lifecycle(resource_json, lifecycle_json)?;
    let (resource, event) = transition(&resource.resourceinstance, &lifecycle, target, strict)?;
    Ok(TransitionResult { resource, event })
}

/// The folded lifecycle state of one resource's layers.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FoldedState {
    pub state_id: Option<String>,
    pub conflict: Vec<String>,
}

/// Fold the stored states of one resource's layers (topmost-first) into one.
///
/// Walks bottom-up. A tile-bearing layer that sets a state must agree with
/// what is below or the result is a Conflict; a layer that sets none defers.
/// A header-only layer (no tiles) is a transition record: its state replaces
/// whatever is below, resolving any Conflict.
pub(crate) fn fold_layer_states(layers: &[(bool, StoredState<'_>)]) -> FoldedState {
    let mut state: Option<&str> = None;
    let mut conflict: Vec<String> = Vec::new();

    for &(has_tiles, stored) in layers.iter().rev() {
        match stored {
            StoredState::Initial => {}
            StoredState::Known(id) if !has_tiles => {
                state = Some(id);
                conflict.clear();
            }
            StoredState::Known(id) => {
                if !conflict.is_empty() {
                    if !conflict.iter().any(|c| c == id) {
                        conflict.push(id.to_string());
                    }
                } else if let Some(prev) = state.filter(|p| *p != id) {
                    conflict = vec![prev.to_string(), id.to_string()];
                    state = None;
                } else {
                    state = Some(id);
                }
            }
            StoredState::Conflict(ids) => {
                if let Some(prev) = state.take() {
                    if !conflict.iter().any(|c| c == prev) {
                        conflict.push(prev.to_string());
                    }
                }
                for id in ids {
                    if !conflict.contains(id) {
                        conflict.push(id.clone());
                    }
                }
            }
        }
    }

    FoldedState {
        state_id: if conflict.is_empty() {
            state.map(str::to_string)
        } else {
            None
        },
        conflict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DRAFT: &str = "9375c9a7-dad2-4f14-a5c1-d7e329fdde4f";
    const ACTIVE: &str = "f75bb034-36e3-4ab4-8167-f520cf0b4c58";
    const RETIRED: &str = "d95d9c0e-0e2c-4450-93a3-d788b91abcc8";

    pub(crate) fn standard() -> serde_json::Value {
        serde_json::json!({
            "id": "7e3cce56-fbfb-4a4b-8e83-59b9f9e7cb75",
            "name": {"en": "Standard"},
            "resource_instance_lifecycle_states": [
                {"action_label": {"en": "Revert to Draft"}, "can_delete_resource_instances": true,
                 "can_edit_resource_instances": true, "id": DRAFT, "is_initial_state": true,
                 "name": {"en": "Draft"}, "next_resource_instance_lifecycle_states": [ACTIVE],
                 "previous_resource_instance_lifecycle_states": [],
                 "resource_instance_lifecycle_id": "7e3cce56-fbfb-4a4b-8e83-59b9f9e7cb75"},
                {"action_label": {"en": "Make Active"}, "can_delete_resource_instances": false,
                 "can_edit_resource_instances": true, "id": ACTIVE, "is_initial_state": false,
                 "name": {"en": "Active"}, "next_resource_instance_lifecycle_states": [RETIRED],
                 "previous_resource_instance_lifecycle_states": [DRAFT],
                 "resource_instance_lifecycle_id": "7e3cce56-fbfb-4a4b-8e83-59b9f9e7cb75"},
                {"action_label": {"en": "Retire"}, "can_delete_resource_instances": false,
                 "can_edit_resource_instances": false, "id": RETIRED, "is_initial_state": false,
                 "name": {"en": "Retired"}, "next_resource_instance_lifecycle_states": [],
                 "previous_resource_instance_lifecycle_states": [ACTIVE],
                 "resource_instance_lifecycle_id": "7e3cce56-fbfb-4a4b-8e83-59b9f9e7cb75"}
            ]
        })
    }

    fn meta(state: Option<&str>) -> StaticResourceMetadata {
        serde_json::from_value(serde_json::json!({
            "descriptors": {},
            "graph_id": "g",
            "name": "r",
            "resourceinstanceid": "r1",
            "resource_instance_lifecycle_state_id": state,
        }))
        .unwrap()
    }

    #[test]
    fn typed_lifecycle_round_trips() {
        let v = standard();
        let typed: StaticLifecycle = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&typed).unwrap(), v);
    }

    #[test]
    fn edges_accept_nested_state_objects() {
        let mut v = standard();
        v["resource_instance_lifecycle_states"][0]["next_resource_instance_lifecycle_states"] =
            serde_json::json!([{"id": ACTIVE, "name": {"en": "Active"}}]);
        let lc = LifecycleIndex::from_value(Some(&v)).unwrap();
        assert!(lc.is_permitted(lc.position(DRAFT).unwrap(), lc.position(ACTIVE).unwrap()));
    }

    #[test]
    fn compile_rejects_malformed_lifecycles() {
        let mut two_initial = standard();
        two_initial["resource_instance_lifecycle_states"][1]["is_initial_state"] = true.into();
        assert!(LifecycleIndex::from_value(Some(&two_initial)).is_err());

        let mut dangling = standard();
        dangling["resource_instance_lifecycle_states"][2]
            ["next_resource_instance_lifecycle_states"] = serde_json::json!(["nowhere"]);
        assert!(LifecycleIndex::from_value(Some(&dangling)).is_err());
    }

    #[test]
    fn graph_compiles_its_exported_lifecycle_and_layers_keep_the_base() {
        let json = include_str!("../../../../tests/data/models/Heritage Item.json");
        let graph = super::super::StaticGraph::from_json_string(json).unwrap();
        let lc = graph.lifecycle().unwrap();
        assert_eq!(lc.initial_state_id(), Some(DRAFT));
        assert_eq!(
            lc.lifecycle_id(),
            Some("7e3cce56-fbfb-4a4b-8e83-59b9f9e7cb75")
        );

        let mut overlay = graph.clone();
        overlay.graphid = "overlay".into();
        overlay.resource_instance_lifecycle = Some(serde_json::json!({
            "id": "other",
            "resource_instance_lifecycle_states": [{"id": "only", "is_initial_state": true}]
        }));
        overlay.invalidate_indices();
        let layered = super::super::LayeredGraph::new(Arc::new(graph), vec![Arc::new(overlay)]);
        assert_eq!(layered.lifecycle().unwrap().initial_state_id(), Some(DRAFT));
        assert_eq!(layered.lifecycle_warnings().len(), 1);
    }

    #[test]
    fn standard_is_a_chain_over_next_and_previous() {
        let lc = LifecycleIndex::from_value(Some(&standard())).unwrap();
        let p = |id| lc.position(id).unwrap();
        assert!(lc.is_permitted(p(DRAFT), p(ACTIVE)));
        assert!(lc.is_permitted(p(ACTIVE), p(DRAFT)));
        assert!(lc.is_permitted(p(ACTIVE), p(RETIRED)));
        assert!(lc.is_permitted(p(RETIRED), p(ACTIVE)));
        assert!(!lc.is_permitted(p(DRAFT), p(RETIRED)));
        assert!(!lc.is_permitted(p(RETIRED), p(DRAFT)));
    }

    #[test]
    fn absent_state_is_initial_and_unknown_is_reported() {
        let lc = LifecycleIndex::from_value(Some(&standard())).unwrap();
        let m = meta(None);
        assert!(matches!(
            lc.effective_state(&m),
            EffectiveState::State {
                id: Some(DRAFT),
                is_initial: true,
                ..
            }
        ));
        let m = meta(Some("nope"));
        assert_eq!(lc.effective_state(&m), EffectiveState::Unknown("nope"));
    }

    #[test]
    fn no_lifecycle_is_perpetual() {
        let lc = LifecycleIndex::from_value(None).unwrap();
        assert!(matches!(
            lc.effective_state(&meta(None)),
            EffectiveState::State {
                id: None,
                is_initial: true,
                ..
            }
        ));
        assert!(transition(&meta(None), &lc, ACTIVE, false).is_err());
    }

    #[test]
    fn strict_transition_follows_edges() {
        let lc = LifecycleIndex::from_value(Some(&standard())).unwrap();
        let (overlay, event) = transition(&meta(None), &lc, ACTIVE, true).unwrap();
        assert!(overlay.tiles.is_none());
        assert_eq!(
            overlay
                .resourceinstance
                .resource_instance_lifecycle_state_id
                .as_deref(),
            Some(ACTIVE)
        );
        assert_eq!(event.from.as_deref(), Some(DRAFT));

        assert!(transition(&meta(None), &lc, RETIRED, true).is_err());
        assert!(transition(&meta(None), &lc, RETIRED, false).is_ok());
        assert!(transition(&meta(None), &lc, "elsewhere", false).is_err());
    }

    #[test]
    fn conflict_needs_explicit_non_strict_transition() {
        let lc = LifecycleIndex::from_value(Some(&standard())).unwrap();
        let mut m = meta(None);
        m.lifecycle_state_conflict = vec![DRAFT.into(), ACTIVE.into()];
        assert!(matches!(
            lc.effective_state(&m),
            EffectiveState::Conflict(_)
        ));
        assert!(transition(&m, &lc, ACTIVE, true).is_err());
        let (overlay, event) = transition(&m, &lc, ACTIVE, false).unwrap();
        assert!(overlay.resourceinstance.lifecycle_state_conflict.is_empty());
        assert_eq!(event.from, None);
        assert_eq!(event.resolved_conflict.len(), 2);
    }

    #[test]
    fn fold_rules() {
        use StoredState::*;
        let known = |s: &str| FoldedState {
            state_id: Some(s.to_string()),
            conflict: vec![],
        };
        // A silent overlay defers to the base.
        assert_eq!(
            fold_layer_states(&[(true, Initial), (true, Known(ACTIVE))]),
            known(ACTIVE)
        );
        // Agreement is fine.
        assert_eq!(
            fold_layer_states(&[(true, Known(ACTIVE)), (true, Known(ACTIVE))]),
            known(ACTIVE)
        );
        // Tile-bearing disagreement is a Conflict.
        let f = fold_layer_states(&[(true, Known(DRAFT)), (true, Known(ACTIVE))]);
        assert_eq!(f.state_id, None);
        assert_eq!(f.conflict, vec![ACTIVE.to_string(), DRAFT.to_string()]);
        // A header-only record on top resolves it.
        assert_eq!(
            fold_layer_states(&[
                (false, Known(RETIRED)),
                (true, Known(DRAFT)),
                (true, Known(ACTIVE)),
            ]),
            known(RETIRED)
        );
        // A tile-bearing layer above a transition record still has to agree.
        let f = fold_layer_states(&[(true, Known(DRAFT)), (false, Known(ACTIVE))]);
        assert_eq!(f.conflict.len(), 2);
        // An already-conflicted layer stays conflicted and unions.
        let conflicted = [DRAFT.to_string(), ACTIVE.to_string()];
        let f = fold_layer_states(&[(true, Known(RETIRED)), (true, Conflict(&conflicted))]);
        assert_eq!(f.conflict.len(), 3);
    }
}
