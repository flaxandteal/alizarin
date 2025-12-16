//! Language configuration for type coercion.

use std::sync::RwLock;

/// Default language when none is set
pub const DEFAULT_LANGUAGE: &str = "en";

lazy_static::lazy_static! {
    /// Global current language setting
    static ref CURRENT_LANGUAGE: RwLock<Option<String>> = RwLock::new(None);
}

/// Get the current language, defaulting to "en" if not set.
pub fn get_current_language() -> String {
    CURRENT_LANGUAGE
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
}

/// Set the current language.
pub fn set_current_language(lang: &str) {
    if let Ok(mut guard) = CURRENT_LANGUAGE.write() {
        *guard = Some(lang.to_string());
    }
}
