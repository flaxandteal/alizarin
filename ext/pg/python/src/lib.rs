use std::collections::HashMap;
use std::sync::Arc;

use rayon::prelude::*;
use serde_json;

use alizarin_core::{
    get_global_extension_registry, get_global_rdm_cache, get_graph,
    RdmCache, StaticGraph,
    DEFAULT_CONFIG_KEYS, DEFAULT_RESOLVABLE_DATATYPES,
};

fn get_registered_graph(graph_id: &str) -> Result<Arc<StaticGraph>, String> {
    get_graph(graph_id).ok_or_else(|| {
        format!(
            "Graph '{}' not registered. Call register_graph() first.",
            graph_id
        )
    })
}

fn build_alias_to_collection_from_graph(
    graph: &StaticGraph,
    ext_registry: Option<&alizarin_core::ExtensionTypeRegistry>,
) -> HashMap<String, String> {
    use std::collections::HashSet;

    let mut resolvable_set: HashSet<&str> = DEFAULT_RESOLVABLE_DATATYPES.iter().copied().collect();

    if let Some(registry) = ext_registry {
        for node in &graph.nodes {
            if registry.has(&node.datatype) {
                resolvable_set.insert(&node.datatype);
            }
        }
    }

    let mut alias_to_collection = HashMap::new();
    for node in &graph.nodes {
        if let Some(alias) = &node.alias {
            if !resolvable_set.contains(node.datatype.as_str()) {
                continue;
            }
            for key in DEFAULT_CONFIG_KEYS {
                if let Some(collection_id) = node.config.get(*key).and_then(|v| v.as_str()) {
                    alias_to_collection.insert(alias.clone(), collection_id.to_string());
                    break;
                }
            }
        }
    }
    alias_to_collection
}

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::PyModule;

    #[pyfunction]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        trees_json,
        graph_id,
        graph_publication_id=None,
        lifecycle_state_id=None,
        created_time=None,
        from_camel=false,
        strict=true,
        id_keys=None,
        resolve_markers=true,
        random_ids=false,
    ))]
    fn batch_trees_to_pg_copy(
        py: Python,
        trees_json: String,
        graph_id: String,
        graph_publication_id: Option<String>,
        lifecycle_state_id: Option<String>,
        created_time: Option<String>,
        from_camel: bool,
        strict: bool,
        id_keys: Option<Vec<String>>,
        resolve_markers: bool,
        random_ids: bool,
    ) -> PyResult<PyObject> {
        let trees: Vec<serde_json::Value> = serde_json::from_str(&trees_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse trees: {}",
                e
            ))
        })?;

        if let Some(ref keys) = id_keys {
            if keys.len() != trees.len() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "id_keys length ({}) must match trees length ({})",
                    keys.len(),
                    trees.len()
                )));
            }
        }

        let graph = get_registered_graph(&graph_id).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(e)
        })?;

        let ext_registry = get_global_extension_registry();

        let mut alias_map =
            build_alias_to_collection_from_graph(&graph, ext_registry.as_ref());
        let label_cache: Option<Arc<RdmCache>> = if alias_map.is_empty() {
            None
        } else {
            get_global_rdm_cache().map(Arc::new)
        };
        if from_camel {
            let camel_entries: Vec<_> = alias_map
                .iter()
                .map(|(k, v)| (alizarin_core::snake_to_camel(k), v.clone()))
                .collect();
            for (k, v) in camel_entries {
                alias_map.insert(k, v);
            }
        }
        let label_lookup = label_cache
            .as_ref()
            .map(|cache| (&alias_map, cache.as_ref()));

        let results: Vec<Result<alizarin_core::StaticResource, String>> = trees
            .into_par_iter()
            .enumerate()
            .map(|(i, tree)| {
                let id_key_ref = id_keys.as_ref().map(|keys| keys[i].as_str());

                alizarin_core::convert_single_tree(
                    &tree,
                    &graph,
                    &graph_id,
                    id_key_ref,
                    from_camel,
                    strict,
                    random_ids,
                    ext_registry.as_ref(),
                    label_lookup,
                    resolve_markers,
                )
                .map_err(|e| format!("Tree {}: {}", i, e))
            })
            .collect();

        let (successes, errors): (Vec<_>, Vec<_>) =
            results.into_iter().partition(Result::is_ok);
        let resources: Vec<alizarin_core::StaticResource> =
            successes.into_iter().map(Result::unwrap).collect();
        let errors: Vec<String> = errors.into_iter().map(|e| e.unwrap_err()).collect();

        if strict && !errors.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Strict mode error: {}",
                errors[0]
            )));
        }

        let tiles_copy = alizarin_pg_core::tiles_to_copy(&resources);
        let resources_copy = alizarin_pg_core::resources_to_copy(
            &resources,
            graph_publication_id.as_deref(),
            lifecycle_state_id.as_deref(),
            created_time.as_deref(),
        );

        let output = serde_json::json!({
            "tiles_copy": tiles_copy,
            "resources_copy": resources_copy,
            "errors": errors,
            "error_count": errors.len(),
        });

        pythonize::pythonize(py, &output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    #[pymodule]
    fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(batch_trees_to_pg_copy, m)?)?;
        Ok(())
    }
}
