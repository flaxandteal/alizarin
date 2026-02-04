//! Translatable string type for multi-language support.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// A string with translations for multiple languages
#[derive(Clone, Debug, Default)]
pub struct StaticTranslatableString {
    pub translations: HashMap<String, String>,
    pub lang: String,
}

impl StaticTranslatableString {
    /// Create a new translatable string from a map of translations
    pub fn from_translations(
        translations: HashMap<String, String>,
        default_lang: Option<String>,
    ) -> Self {
        let lang = default_lang.unwrap_or_else(|| "en".to_string());
        let actual_lang = if translations.contains_key(&lang) {
            lang
        } else {
            translations
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "en".to_string())
        };
        StaticTranslatableString {
            translations,
            lang: actual_lang,
        }
    }

    /// Create from a simple string (assumes English)
    pub fn from_string(s: &str) -> Self {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), s.to_string());
        StaticTranslatableString {
            translations,
            lang: "en".to_string(),
        }
    }

    /// Create an empty translatable string
    pub fn empty() -> Self {
        StaticTranslatableString {
            translations: HashMap::new(),
            lang: "en".to_string(),
        }
    }

    /// Get the string for a specific language, falling back to any available
    pub fn get(&self, lang: &str) -> String {
        self.translations
            .get(lang)
            .or_else(|| self.translations.get("en"))
            .or_else(|| self.translations.values().next())
            .cloned()
            .unwrap_or_default()
    }

    /// Get the string using the default language
    pub fn to_string_default(&self) -> String {
        self.get(&self.lang)
    }

    /// Copy/clone the translatable string
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.translations).unwrap_or(serde_json::Value::Null)
    }
}

impl std::fmt::Display for StaticTranslatableString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_default())
    }
}

impl Serialize for StaticTranslatableString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.translations.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StaticTranslatableString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_json::Value;
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Object(map) => {
                let translations: HashMap<String, String> = map
                    .into_iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                    .collect();

                let lang = translations
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "en".to_string());

                Ok(StaticTranslatableString { translations, lang })
            }
            Value::String(s) => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), s);
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
            _ => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), String::new());
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translatable_string_from_map() {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), "Hello".to_string());
        translations.insert("de".to_string(), "Hallo".to_string());

        let ts = StaticTranslatableString::from_translations(translations, None);
        assert_eq!(ts.get("en"), "Hello");
        assert_eq!(ts.get("de"), "Hallo");
    }

    #[test]
    fn test_translatable_string_fallback() {
        let mut translations = HashMap::new();
        translations.insert("de".to_string(), "Hallo".to_string());

        let ts = StaticTranslatableString::from_translations(translations, None);
        // Should fall back to any available language
        assert_eq!(ts.get("en"), "Hallo");
    }
}
