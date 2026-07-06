use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

/// The cldr-json release tag the built-in workflow is pinned to.
pub const CLDR_JSON_VERSION: &str = "47.0.0";

/// Returns the pinned cldr-json URLs for one locale's display names.
pub fn built_in_download(lang: &str) -> (String, Vec<String>) {
    let base = format!(
        "https://raw.githubusercontent.com/unicode-org/cldr-json/{CLDR_JSON_VERSION}/cldr-json/cldr-localenames-full/main/{lang}"
    );
    let urls = vec![
        format!("{base}/territories.json"),
        format!("{base}/languages.json"),
    ];
    (base, urls)
}

/// Merges the fetched CLDR documents into one JSON array payload.
pub fn merge(downloads: Vec<Vec<u8>>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut docs = Vec::with_capacity(downloads.len());
    for bytes in downloads {
        docs.push(serde_json::from_slice::<Value>(&bytes)?);
    }
    Ok(serde_json::to_vec(&Value::Array(docs))?)
}

/// Parses CLDR locale display names (territories and languages).
///
/// Every display value becomes its own canonical record; the map keys are
/// locale codes ("DE", "fr", "US-alt-short"), not text, so they are never
/// used as aliases.
pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let docs: Vec<&Value> = match &value {
        Value::Array(items) => items.iter().collect(),
        _ => vec![&value],
    };

    let mut names = BTreeSet::new();
    for doc in docs {
        let Some(locales) = doc.get("main").and_then(Value::as_object) else {
            continue;
        };
        for locale_doc in locales.values() {
            let Some(display_names) = locale_doc
                .get("localeDisplayNames")
                .and_then(Value::as_object)
            else {
                continue;
            };
            for section in ["territories", "languages"] {
                let Some(entries) = display_names.get(section).and_then(Value::as_object) else {
                    continue;
                };
                for (code, display) in entries {
                    let Some(display) = display.as_str() else {
                        continue;
                    };
                    let display = display.trim();
                    // Skip untranslated placeholders that just echo the code.
                    if display.is_empty()
                        || display == code
                        || !display.chars().any(char::is_alphabetic)
                    {
                        continue;
                    }
                    names.insert(display.to_string());
                }
            }
        }
    }

    if names.is_empty() {
        return Err("CLDR payload did not yield any display names".into());
    }
    Ok(names
        .into_iter()
        .map(|canonical| SourceRecord {
            canonical,
            aliases: Vec::new(),
            score: 2.0,
        })
        .collect())
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    parse(bytes)?;
    Ok(())
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("de");
    serde_json::to_vec_pretty(&serde_json::json!({
        "main": {
            locale: {
                "localeDisplayNames": {
                    "territories": {
                        "DE": "Deutschland",
                        "US": "Vereinigte Staaten",
                        "US-alt-short": "USA",
                        "ZZ": "ZZ"
                    },
                    "languages": {
                        "de": "Deutsch",
                        "fr": "Französisch"
                    }
                }
            }
        }
    }))
    .expect("sample serialization should succeed")
}

#[cfg(test)]
mod tests {
    use super::{merge, parse, sample};

    #[test]
    fn parses_territory_and_language_display_names() {
        let records = parse(&sample(Some("de"))).expect("parse sample");
        let names: Vec<&str> = records
            .iter()
            .map(|record| record.canonical.as_str())
            .collect();
        assert!(names.contains(&"Deutschland"));
        assert!(names.contains(&"Vereinigte Staaten"));
        assert!(names.contains(&"USA"));
        assert!(names.contains(&"Französisch"));
        // The untranslated placeholder that echoes its code is skipped.
        assert!(!names.contains(&"ZZ"));
    }

    #[test]
    fn merges_multiple_documents() {
        let merged = merge(vec![sample(Some("de")), sample(Some("sv"))]).expect("merge");
        let records = parse(&merged).expect("parse merged");
        assert!(!records.is_empty());
    }
}
