use super::{SourceRecord, parse_json_records, sample_json};

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_json_records(bytes)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    sample_json(
        "musicbrainz",
        locale,
        &[
            ("Björk", &["bjork"]),
            ("The Beatles", &["beatles"]),
            ("Kind of Blue", &["kind of blue"]),
        ],
    )
}
