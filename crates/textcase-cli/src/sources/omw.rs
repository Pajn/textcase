use super::{SourceRecord, parse_json_records, sample_json};

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_json_records(bytes)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    sample_json(
        "omw",
        locale,
        &[("Sprache", &["sprache"]), ("Analyse", &["analyse"])],
    )
}
