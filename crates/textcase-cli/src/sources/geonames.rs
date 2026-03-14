use super::{SourceRecord, parse_delimited_records};

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_delimited_records(bytes, '\t')
}

pub fn sample(country: Option<&str>) -> Vec<u8> {
    let country = country.unwrap_or("XX");
    format!(
        "name	alt_name	score
Berlin	berlin	2
Munich	munich	1.5
Country {country}	country {country}	1
"
    )
    .into_bytes()
}
