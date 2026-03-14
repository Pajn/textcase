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
        Value::Object(object) => {
            if let Some(Value::Array(items)) = object.get("member") {
                for item in items {
                    if let Some(record) = parse_record(item) {
                        records.push(record);
                    }
                }
            } else if let Some(record) = parse_record(&value) {
                records.push(record);
            }
        }
        _ => {}
    }

    if records.is_empty() {
        return Err("GND payload did not yield any records".into());
    }
    Ok(records)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("de");
    serde_json::to_vec_pretty(&serde_json::json!({
        "id": "https://d-nb.info/gnd/118540238",
        "type": ["AuthorityResource", "DifferentiatedPerson", "Person"],
        "preferredName": "Goethe, Johann Wolfgang von",
        "preferredNameEntityForThePerson": {
            "forename": ["Johann Wolfgang"],
            "prefix": ["von"],
            "surname": ["Goethe"]
        },
        "variantName": ["Johann Wolfgang von Goethe", "J. W. von Goethe"],
        "variantNameEntityForThePerson": [
            { "forename": ["Johann Wolfgang"], "surname": ["Goethe"] },
            { "personalName": ["Goethe"] }
        ],
        "originalScriptNameOfThePerson": [
            { "@language": locale, "@value": "Johann Wolfgang von Goethe" }
        ]
    }))
    .expect("sample serialization should succeed")
}

fn parse_record(value: &Value) -> Option<SourceRecord> {
    let object = value.as_object()?;
    let canonical = object
        .get("preferredName")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| parse_name_entity(object.get("preferredNameEntityForThePerson")))?;

    let mut aliases = BTreeSet::new();
    collect_string_values(object.get("variantName"), &mut aliases);
    collect_name_entities(object.get("variantNameEntityForThePerson"), &mut aliases);
    collect_language_values(object.get("originalScriptNameOfThePerson"), &mut aliases);
    aliases.remove(&canonical);

    let type_score = object
        .get("type")
        .and_then(Value::as_array)
        .map(|types| {
            if types.iter().any(|value| value.as_str() == Some("Person")) {
                2.0
            } else if types
                .iter()
                .any(|value| value.as_str() == Some("PlaceOrGeographicName"))
            {
                1.8
            } else {
                1.2
            }
        })
        .unwrap_or(1.0);

    Some(SourceRecord {
        canonical,
        aliases: aliases.into_iter().collect(),
        score: type_score,
    })
}

fn collect_string_values(value: Option<&Value>, aliases: &mut BTreeSet<String>) {
    let Some(value) = value else {
        return;
    };
    match value {
        Value::String(value) => {
            aliases.insert(value.to_string());
        }
        Value::Array(values) => {
            for value in values {
                collect_string_values(Some(value), aliases);
            }
        }
        _ => {}
    }
}

fn collect_language_values(value: Option<&Value>, aliases: &mut BTreeSet<String>) {
    let Some(Value::Array(values)) = value else {
        return;
    };
    for value in values {
        if let Some(entry) = value.get("@value").and_then(Value::as_str) {
            aliases.insert(entry.to_string());
        }
    }
}

fn collect_name_entities(value: Option<&Value>, aliases: &mut BTreeSet<String>) {
    let Some(value) = value else {
        return;
    };
    match value {
        Value::Array(values) => {
            for value in values {
                if let Some(alias) = parse_name_entity(Some(value)) {
                    aliases.insert(alias);
                }
            }
        }
        _ => {
            if let Some(alias) = parse_name_entity(Some(value)) {
                aliases.insert(alias);
            }
        }
    }
}

fn parse_name_entity(value: Option<&Value>) -> Option<String> {
    let object = value?.as_object()?;
    if let Some(Value::Array(personal_names)) = object.get("personalName") {
        return personal_names
            .iter()
            .find_map(Value::as_str)
            .map(str::to_string);
    }

    let forename = object
        .get("forename")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    let prefix = object
        .get("prefix")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    let surname = object
        .get("surname")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" ");

    let mut parts = Vec::new();
    if !forename.is_empty() {
        parts.push(forename);
    }
    if !prefix.is_empty() {
        parts.push(prefix);
    }
    if !surname.is_empty() {
        parts.push(surname);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_lobid_style_gnd_records() {
        let records = parse(&sample(Some("de"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Goethe, Johann Wolfgang von");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Johann Wolfgang von Goethe")
        );
    }
}
