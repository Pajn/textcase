use std::collections::BTreeSet;

use serde_json::Value;

use super::SourceRecord;

/// The natural-earth-vector release tag the built-in workflow is pinned to.
pub const NATURAL_EARTH_VERSION: &str = "v5.1.2";

/// Returns the pinned GeoJSON URLs: world countries at 110m plus populated
/// places (simple attributes) at 10m.
pub fn built_in_download() -> (String, Vec<String>) {
    let base = format!(
        "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/{NATURAL_EARTH_VERSION}/geojson"
    );
    let urls = vec![
        format!("{base}/ne_110m_admin_0_countries.geojson"),
        format!("{base}/ne_10m_populated_places_simple.geojson"),
    ];
    (base, urls)
}

/// Merges the fetched FeatureCollections into one.
pub fn merge(downloads: Vec<Vec<u8>>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut features = Vec::new();
    for bytes in downloads {
        let value: Value = serde_json::from_slice(&bytes)?;
        let items = value
            .get("features")
            .and_then(Value::as_array)
            .ok_or("Natural Earth payload is not a GeoJSON FeatureCollection")?;
        features.extend(items.iter().cloned());
    }
    Ok(serde_json::to_vec(&serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    }))?)
}

/// Parses a Natural Earth GeoJSON FeatureCollection.
///
/// Handles both attribute spellings: admin-0 country features use uppercase
/// keys (`NAME`, `NAME_LONG`, `ADMIN`), populated places lowercase ones
/// (`name`, `nameascii`, `namealt`).
pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let features = value
        .get("features")
        .and_then(Value::as_array)
        .ok_or("Natural Earth payload is not a GeoJSON FeatureCollection")?;

    let mut records = Vec::new();
    for feature in features {
        let Some(properties) = feature.get("properties").and_then(Value::as_object) else {
            continue;
        };
        let text = |key: &str| -> Option<&str> { properties.get(key).and_then(Value::as_str) };

        let mut aliases = BTreeSet::new();
        let (canonical, score) = if let Some(name) = text("NAME") {
            // Country feature.
            for key in ["NAME_LONG", "ADMIN"] {
                if let Some(alias) = text(key)
                    && !alias.is_empty()
                    && alias != name
                {
                    aliases.insert(alias.to_string());
                }
            }
            (name.to_string(), 2.8)
        } else if let Some(name) = text("name") {
            // Populated place feature.
            for key in ["nameascii", "namealt"] {
                if let Some(alias) = text(key)
                    && !alias.is_empty()
                    && alias != name
                {
                    aliases.insert(alias.to_string());
                }
            }
            let population = properties
                .get("pop_max")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let population_bonus = if population > 0.0 {
                (population.log10() / 10.0) as f32
            } else {
                0.0
            };
            (name.to_string(), 2.0 + population_bonus)
        } else {
            continue;
        };

        if canonical.trim().is_empty() {
            continue;
        }
        records.push(SourceRecord {
            canonical,
            aliases: aliases.into_iter().collect(),
            score,
        });
    }

    if records.is_empty() {
        return Err("Natural Earth payload did not yield any records".into());
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    parse(bytes)?;
    Ok(())
}

pub fn sample(_locale: Option<&str>) -> Vec<u8> {
    serde_json::to_vec_pretty(&serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {
                    "featurecla": "Admin-0 country",
                    "NAME": "Côte d'Ivoire",
                    "NAME_LONG": "Ivory Coast",
                    "ADMIN": "Ivory Coast",
                    "POP_EST": 25716544
                }
            },
            {
                "type": "Feature",
                "properties": {
                    "featurecla": "Populated place",
                    "name": "São Paulo",
                    "nameascii": "Sao Paulo",
                    "namealt": null,
                    "pop_max": 18845000
                }
            }
        ]
    }))
    .expect("sample serialization should succeed")
}

#[cfg(test)]
mod tests {
    use super::{merge, parse, sample};

    #[test]
    fn parses_countries_and_populated_places() {
        let records = parse(&sample(None)).expect("parse sample");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Côte d'Ivoire");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Ivory Coast")
        );
        assert_eq!(records[1].canonical, "São Paulo");
        assert!(records[1].aliases.iter().any(|alias| alias == "Sao Paulo"));
        // A large city outranks the base score but not a country.
        assert!(records[1].score > 2.0);
        assert!(records[0].score > records[1].score);
    }

    #[test]
    fn skips_empty_string_aliases() {
        let payload = serde_json::to_vec(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": { "name": "Springfield", "nameascii": "", "namealt": "Spring City" }
            }]
        }))
        .unwrap();
        let records = parse(&payload).expect("parse");
        assert_eq!(records[0].canonical, "Springfield");
        // The empty nameascii is dropped; the real namealt survives.
        assert_eq!(records[0].aliases, vec!["Spring City".to_string()]);
    }

    #[test]
    fn merges_feature_collections() {
        let merged = merge(vec![sample(None), sample(None)]).expect("merge");
        let records = parse(&merged).expect("parse merged");
        assert_eq!(records.len(), 4);
    }
}
