use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let mut records = Vec::new();

    match &value {
        Value::Array(items) => {
            for item in items {
                if let Some(record) = parse_record(item) {
                    records.push(record);
                }
            }
        }
        Value::Object(_) => {
            if let Some(record) = parse_record(&value) {
                records.push(record);
            }
        }
        _ => {}
    }

    if records.is_empty() {
        return Err("Getty payload did not yield any records".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("en");
    serde_json::to_vec_pretty(&serde_json::json!({
        "@context": "https://linked.art/ns/v1/linked-art.json",
        "_label": "Gogh, Vincent van",
        "identified_by": [
            { "content": "Vincent van Gogh", "language": locale },
            { "content": "van Gogh, Vincent", "language": locale }
        ]
    }))
    .expect("sample serialization should succeed")
}

fn parse_record(value: &Value) -> Option<SourceRecord> {
    let object = value.as_object()?;
    let canonical = object
        .get("_label")
        .and_then(Value::as_str)
        .or_else(|| {
            object
                .get("identified_by")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items
                        .iter()
                        .find_map(|item| item.get("content").and_then(Value::as_str))
                })
        })?
        .to_string();

    let mut aliases = BTreeSet::new();
    if let Some(Value::Array(items)) = object.get("identified_by") {
        for item in items {
            if let Some(alias) = item.get("content").and_then(Value::as_str) {
                aliases.insert(alias.to_string());
            }
        }
    }
    aliases.remove(&canonical);

    Some(SourceRecord {
        canonical,
        aliases: aliases.into_iter().collect(),
        score: 1.5,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_getty_linked_art_json() {
        let records = parse(&sample(Some("en"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Gogh, Vincent van");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Vincent van Gogh")
        );
    }
}
