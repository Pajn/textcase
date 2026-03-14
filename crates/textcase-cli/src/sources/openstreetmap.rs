use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let items = value
        .as_array()
        .ok_or("OpenStreetMap payload must be a JSON array")?;

    let mut records = Vec::new();
    for item in items {
        let Some(object) = item.as_object() else {
            continue;
        };
        let canonical = object
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| {
                object
                    .get("display_name")
                    .and_then(Value::as_str)
                    .and_then(|value| value.split(',').next())
                    .map(str::trim)
            })
            .ok_or("OpenStreetMap result is missing a name")?;

        let mut aliases = BTreeSet::new();
        if let Some(display_name) = object.get("display_name").and_then(Value::as_str) {
            aliases.insert(display_name.to_string());
        }
        if let Some(namedetails) = object.get("namedetails").and_then(Value::as_object) {
            for value in namedetails.values() {
                if let Some(value) = value.as_str() {
                    aliases.insert(value.to_string());
                }
            }
        }
        aliases.remove(canonical);

        let importance = object
            .get("importance")
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        let place_rank = object
            .get("place_rank")
            .and_then(Value::as_u64)
            .unwrap_or(0) as f32;

        records.push(SourceRecord {
            canonical: canonical.to_string(),
            aliases: aliases.into_iter().collect(),
            score: 1.0 + importance + (place_rank / 100.0),
        });
    }

    if records.is_empty() {
        return Err("OpenStreetMap payload did not yield any records".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("de");
    serde_json::to_vec_pretty(&serde_json::json!([
        {
            "name": "Berlin",
            "display_name": "Berlin, Deutschland",
            "importance": 0.85,
            "place_rank": 8,
            "namedetails": {
                "name": "Berlin",
                "name:en": "Berlin",
                (format!("name:{locale}")): "Berlin"
            }
        }
    ]))
    .expect("sample serialization should succeed")
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_nominatim_style_json() {
        let records = parse(&sample(Some("de"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Berlin");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Berlin, Deutschland")
        );
    }
}
