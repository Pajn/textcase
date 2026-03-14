use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let mut records = Vec::new();

    match &value {
        Value::Object(object) => {
            for key in ["artists", "releases", "works", "recordings"] {
                if let Some(Value::Array(items)) = object.get(key) {
                    for item in items {
                        if let Some(record) = parse_record(item) {
                            records.push(record);
                        }
                    }
                }
            }
            if records.is_empty()
                && let Some(record) = parse_record(&value)
            {
                records.push(record);
            }
        }
        Value::Array(items) => {
            for item in items {
                if let Some(record) = parse_record(item) {
                    records.push(record);
                }
            }
        }
        _ => {}
    }

    if records.is_empty() {
        return Err("MusicBrainz payload did not yield any records".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("en");
    serde_json::to_vec_pretty(&serde_json::json!({
        "artists": [
            {
                "name": "Björk",
                "sort-name": "Bjork",
                "aliases": [
                    { "name": "bjork", "locale": locale },
                    { "name": "Björk Guðmundsdóttir", "locale": locale }
                ]
            },
            {
                "name": "Kind of Blue",
                "aliases": [{ "name": "kind of blue", "locale": locale }]
            }
        ]
    }))
    .expect("sample serialization should succeed")
}

fn parse_record(value: &Value) -> Option<SourceRecord> {
    let object = value.as_object()?;
    let canonical = object
        .get("name")
        .or_else(|| object.get("title"))
        .and_then(Value::as_str)?
        .to_string();

    let mut aliases = BTreeSet::new();
    if let Some(sort_name) = object.get("sort-name").and_then(Value::as_str) {
        aliases.insert(sort_name.to_string());
    }
    if let Some(Value::Array(items)) = object.get("aliases") {
        for item in items {
            if let Some(alias) = item.get("name").and_then(Value::as_str) {
                aliases.insert(alias.to_string());
            }
        }
    }
    if let Some(disambiguation) = object.get("disambiguation").and_then(Value::as_str)
        && !disambiguation.is_empty()
    {
        aliases.insert(disambiguation.to_string());
    }
    aliases.remove(&canonical);

    Some(SourceRecord {
        canonical: canonical.clone(),
        aliases: aliases.into_iter().collect(),
        score: if is_mixed_case_brand(&canonical) {
            2.0
        } else {
            1.2
        },
    })
}

fn is_mixed_case_brand(value: &str) -> bool {
    value.chars().any(|char| char.is_lowercase()) && value.chars().any(|char| char.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_musicbrainz_result_sets() {
        let records = parse(&sample(Some("en"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Björk");
        assert!(records[0].aliases.iter().any(|alias| alias == "Bjork"));
    }
}
