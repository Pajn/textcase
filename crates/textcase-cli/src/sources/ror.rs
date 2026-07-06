use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
};

use serde_json::Value;
use zip::ZipArchive;

use super::SourceRecord;

/// Parses a ROR registry data dump (schema v1 or v2 JSON).
///
/// Acronyms are deliberately excluded from the aliases: mapping "mit" to
/// "Massachusetts Institute of Technology" would rewrite ordinary words, not
/// restore casing.
pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let items = value
        .as_array()
        .ok_or("ROR payload is not a JSON array of organizations")?;

    let mut records = Vec::new();
    for item in items {
        if let Some(record) = parse_record(item) {
            records.push(record);
        }
    }

    if records.is_empty() {
        return Err("ROR payload did not yield any records".into());
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    parse(bytes)?;
    Ok(())
}

/// ROR publishes the dump as a Zenodo zip holding v1 and v2 JSON files; the
/// v2 schema is preferred. A plain JSON payload passes through.
pub fn extract_payload(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !bytes.starts_with(b"PK") {
        return Ok(bytes.to_vec());
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let names: Vec<String> = archive.file_names().map(str::to_owned).collect();
    let chosen = names
        .iter()
        .find(|name| name.ends_with(".json") && name.contains("schema_v2"))
        .or_else(|| names.iter().find(|name| name.ends_with(".json")))
        .ok_or("ROR archive did not contain a .json payload")?
        .clone();
    let mut file = archive.by_name(&chosen)?;
    let mut out = Vec::new();
    file.read_to_end(&mut out)?;
    Ok(out)
}

pub fn sample(_locale: Option<&str>) -> Vec<u8> {
    serde_json::to_vec_pretty(&serde_json::json!([
        {
            "id": "https://ror.org/05f950310",
            "names": [
                { "value": "Karolinska Institutet", "types": ["ror_display", "label"], "lang": "sv" },
                { "value": "Karolinska Institute", "types": ["label"], "lang": "en" },
                { "value": "KI", "types": ["acronym"], "lang": null }
            ]
        },
        {
            "id": "https://ror.org/03vek6s52",
            "name": "Harvard University",
            "aliases": ["Harvard"],
            "labels": [{ "label": "Université Harvard", "iso639": "fr" }],
            "acronyms": ["HU"]
        }
    ]))
    .expect("sample serialization should succeed")
}

fn parse_record(value: &Value) -> Option<SourceRecord> {
    let object = value.as_object()?;

    let mut canonical: Option<String> = None;
    let mut aliases = BTreeSet::new();

    if let Some(Value::Array(names)) = object.get("names") {
        // Schema v2: one names list with typed entries.
        for name in names {
            let Some(text) = name.get("value").and_then(Value::as_str) else {
                continue;
            };
            let types: Vec<&str> = name
                .get("types")
                .and_then(Value::as_array)
                .map(|types| types.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if types.contains(&"acronym") {
                continue;
            }
            if types.contains(&"ror_display") && canonical.is_none() {
                canonical = Some(text.to_string());
            } else {
                aliases.insert(text.to_string());
            }
        }
    } else {
        // Schema v1.
        canonical = object
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if let Some(Value::Array(items)) = object.get("aliases") {
            for alias in items.iter().filter_map(Value::as_str) {
                aliases.insert(alias.to_string());
            }
        }
        if let Some(Value::Array(items)) = object.get("labels") {
            for label in items {
                if let Some(text) = label.get("label").and_then(Value::as_str) {
                    aliases.insert(text.to_string());
                }
            }
        }
    }

    let canonical = canonical?;
    aliases.remove(&canonical);
    Some(SourceRecord {
        canonical,
        aliases: aliases.into_iter().collect(),
        score: 1.8,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{extract_payload, parse, sample};

    #[test]
    fn parses_v2_and_v1_records_without_acronyms() {
        let records = parse(&sample(None)).expect("parse sample");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Karolinska Institutet");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Karolinska Institute")
        );
        assert!(!records[0].aliases.iter().any(|alias| alias == "KI"));
        assert_eq!(records[1].canonical, "Harvard University");
        assert!(
            records[1]
                .aliases
                .iter()
                .any(|alias| alias == "Université Harvard")
        );
        assert!(!records[1].aliases.iter().any(|alias| alias == "HU"));
    }

    #[test]
    fn extracts_v2_json_from_zip() {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file::<_, ()>("v1.63-ror-data.json", zip::write::FileOptions::default())
            .expect("start v1 file");
        writer.write_all(b"[]").expect("write v1");
        writer
            .start_file::<_, ()>(
                "v1.63-ror-data_schema_v2.json",
                zip::write::FileOptions::default(),
            )
            .expect("start v2 file");
        writer.write_all(&sample(None)).expect("write v2");
        let cursor = writer.finish().expect("finish zip");
        let extracted = extract_payload(cursor.get_ref()).expect("extract");
        assert_eq!(extracted, sample(None));
    }
}
