use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let mut records = parse_lookup_results(&value);
    if records.is_empty() {
        records = parse_resource_graph(&value);
    }
    if records.is_empty() {
        return Err("DBpedia payload did not yield any records".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("en");
    serde_json::to_vec_pretty(&serde_json::json!({
        "docs": [
            {
                "label": ["<B>Berlin</B>"],
                "redirectlabel": ["Berlin City", "Berolinum"],
                "score": ["45580.38"],
                "resource": ["http://dbpedia.org/resource/Berlin"],
                "language": [locale]
            }
        ]
    }))
    .expect("sample serialization should succeed")
}

fn parse_lookup_results(value: &Value) -> Vec<SourceRecord> {
    let Some(items) = value.get("docs").and_then(Value::as_array) else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            let canonical = item
                .get("label")
                .and_then(Value::as_array)
                .and_then(|values| values.first())
                .and_then(Value::as_str)
                .map(strip_highlight_tags)?;

            let mut aliases = BTreeSet::new();
            if let Some(values) = item.get("redirectlabel").and_then(Value::as_array) {
                for value in values {
                    if let Some(value) = value.as_str() {
                        aliases.insert(strip_highlight_tags(value));
                    }
                }
            }

            let score = item
                .get("score")
                .and_then(Value::as_array)
                .and_then(|values| values.first())
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(1.0);

            aliases.remove(&canonical);
            Some(SourceRecord {
                canonical,
                aliases: aliases.into_iter().collect(),
                score,
            })
        })
        .collect()
}

fn parse_resource_graph(value: &Value) -> Vec<SourceRecord> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };

    let mut records = Vec::new();
    for predicates in object.values() {
        let Some(predicates) = predicates.as_object() else {
            continue;
        };

        let mut labels = BTreeSet::new();
        let mut aliases = BTreeSet::new();
        for (predicate, values) in predicates {
            let Some(values) = values.as_array() else {
                continue;
            };
            for value in values {
                let Some(text) = value.get("value").and_then(Value::as_str) else {
                    continue;
                };
                if predicate.ends_with("label") || predicate.ends_with("name") {
                    labels.insert(text.to_string());
                } else if predicate.ends_with("alias") || predicate.ends_with("redirectlabel") {
                    aliases.insert(text.to_string());
                }
            }
        }
        let Some(canonical) = labels.into_iter().next() else {
            continue;
        };
        aliases.remove(&canonical);
        records.push(SourceRecord {
            canonical,
            aliases: aliases.into_iter().collect(),
            score: 1.0,
        });
    }
    records
}

fn strip_highlight_tags(value: &str) -> String {
    value
        .replace("<B>", "")
        .replace("</B>", "")
        .replace("<b>", "")
        .replace("</b>", "")
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_dbpedia_lookup_json() {
        let records = parse(&sample(Some("en"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Berlin");
        assert!(records[0].aliases.iter().any(|alias| alias == "Berolinum"));
    }
}
