use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

pub fn parse(
    bytes: &[u8],
    lang: Option<&str>,
) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let entities = value
        .get("entities")
        .and_then(Value::as_object)
        .ok_or("Wikidata payload is missing an entities object")?;

    let mut records = Vec::new();
    for entity in entities.values() {
        if let Some(record) = parse_entity(entity, lang) {
            records.push(record);
        }
    }
    if records.is_empty() {
        return Err("Wikidata payload did not yield any entities".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let lang = locale.unwrap_or("en");
    serde_json::to_vec_pretty(&serde_json::json!({
        "entities": {
            "Q64": {
                "id": "Q64",
                "labels": {
                    lang: { "language": lang, "value": "Berlin" },
                    "de": { "language": "de", "value": "Berlin" }
                },
                "aliases": {
                    lang: [
                        { "language": lang, "value": "berlin" },
                        { "language": lang, "value": "Berlin City" }
                    ]
                }
            },
            "Q60": {
                "id": "Q60",
                "labels": {
                    lang: { "language": lang, "value": "New York City" }
                },
                "aliases": {
                    lang: [
                        { "language": lang, "value": "New York" },
                        { "language": lang, "value": "NYC" }
                    ]
                }
            }
        }
    }))
    .expect("sample serialization should succeed")
}

fn parse_entity(entity: &Value, lang: Option<&str>) -> Option<SourceRecord> {
    let labels = entity.get("labels")?.as_object()?;
    let canonical = preferred_wikidata_value(labels, lang)?;
    let mut aliases = BTreeSet::new();

    if let Some(aliases_by_lang) = entity.get("aliases").and_then(Value::as_object) {
        if let Some(lang) = lang {
            collect_wikidata_aliases(aliases_by_lang.get(lang), &mut aliases);
        } else {
            for values in aliases_by_lang.values() {
                collect_wikidata_aliases(Some(values), &mut aliases);
            }
        }
    }

    if let Some(lang) = lang {
        for other in labels.values() {
            if let Some(value) = other.get("value").and_then(Value::as_str)
                && value != canonical
                && other.get("language").and_then(Value::as_str) == Some(lang)
            {
                aliases.insert(value.to_string());
            }
        }
    }

    Some(SourceRecord {
        canonical: canonical.clone(),
        aliases: aliases.into_iter().collect(),
        score: if canonical.contains(' ') { 1.5 } else { 1.0 },
    })
}

fn preferred_wikidata_value(
    labels: &serde_json::Map<String, Value>,
    lang: Option<&str>,
) -> Option<String> {
    lang.and_then(|lang| labels.get(lang))
        .or_else(|| labels.get("en"))
        .or_else(|| labels.get("de"))
        .or_else(|| labels.values().next())
        .and_then(|label| label.get("value"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn collect_wikidata_aliases(value: Option<&Value>, aliases: &mut BTreeSet<String>) {
    let Some(Value::Array(entries)) = value else {
        return;
    };
    for entry in entries {
        if let Some(alias) = entry.get("value").and_then(Value::as_str) {
            aliases.insert(alias.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_wikidata_entity_json() {
        let records = parse(&sample(Some("en")), Some("en")).expect("parse sample");
        assert!(records.iter().any(|record| record.canonical == "Berlin"));
        assert!(
            records
                .iter()
                .any(|record| record.aliases.iter().any(|alias| alias == "NYC"))
        );
    }
}
