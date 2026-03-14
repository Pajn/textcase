use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
};

use zip::ZipArchive;

use super::SourceRecord;

const GEONAMES_MIN_COLUMNS: usize = 19;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let text = std::str::from_utf8(bytes)?;
    let mut records = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < GEONAMES_MIN_COLUMNS {
            return Err(format!(
                "expected at least {GEONAMES_MIN_COLUMNS} GeoNames columns, got {}",
                fields.len()
            )
            .into());
        }

        let canonical = fields[1].trim();
        if canonical.is_empty() {
            continue;
        }

        let mut aliases = BTreeSet::new();
        let asciiname = fields[2].trim();
        if !asciiname.is_empty() && asciiname != canonical {
            aliases.insert(asciiname.to_string());
        }
        for alias in fields[3].split(',') {
            let alias = alias.trim();
            if !alias.is_empty() && alias != canonical {
                aliases.insert(alias.to_string());
            }
        }

        let feature_class = fields[6].trim();
        let feature_code = fields[7].trim();
        let population = fields[14].trim().parse::<u64>().unwrap_or(0);

        records.push(SourceRecord {
            canonical: canonical.to_string(),
            aliases: aliases.into_iter().collect(),
            score: geonames_score(feature_class, feature_code, population),
        });
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let records = parse(bytes)?;
    if records.is_empty() {
        return Err("GeoNames payload did not yield any records".into());
    }
    Ok(())
}

pub fn extract_zip(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        if !file.name().ends_with(".txt") {
            continue;
        }
        let mut out = Vec::new();
        file.read_to_end(&mut out)?;
        if looks_like_geonames_payload(&out) {
            return Ok(out);
        }
    }
    Err("GeoNames archive did not contain a .txt payload".into())
}

pub fn sample(country: Option<&str>) -> Vec<u8> {
    let country = country.unwrap_or("XX");
    format!(
        "2950159\tBerlin\tBerlin\tBerlin,Berlim\t52.52437\t13.41053\tP\tPPLC\tDE\t\t16\t00\t00\t00\t3644826\t34\t74\tEurope/Berlin\t2024-01-01
2867714\tMuenchen\tMuenchen\tMunich,München\t48.13743\t11.57549\tP\tPPLA\t{country}\t\t02\t00\t00\t00\t1260391\t519\t518\tEurope/Berlin\t2024-01-01
"
    )
    .into_bytes()
}

fn geonames_score(feature_class: &str, feature_code: &str, population: u64) -> f32 {
    let class_score = match feature_class {
        "P" => 3.0,
        "A" => 2.6,
        "S" => 2.2,
        "L" | "T" => 1.8,
        _ => 1.2,
    };
    let code_bonus = match feature_code {
        "PPLC" => 1.0,
        "PPLA" | "PPLA2" | "PPLA3" | "PPLA4" => 0.6,
        "PPL" => 0.3,
        _ => 0.0,
    };
    let population_bonus = if population == 0 {
        0.0
    } else {
        ((population as f64).log10() / 10.0) as f32
    };
    class_score + code_bonus + population_bonus
}

fn looks_like_geonames_payload(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    text.lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.split('\t').count() >= GEONAMES_MIN_COLUMNS)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::{extract_zip, parse, sample};

    #[test]
    fn parses_official_geonames_rows() {
        let records = parse(&sample(Some("DE"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Berlin");
        assert!(records[0].aliases.iter().any(|alias| alias == "Berlim"));
        assert!(records[0].score > records[1].score);
    }

    #[test]
    fn extracts_text_payload_from_zip_archive() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file::<_, ()>("DE.txt", zip::write::FileOptions::default())
            .expect("start file");
        writer.write_all(&sample(Some("DE"))).expect("write sample");
        let cursor = writer.finish().expect("finish zip");
        let extracted = extract_zip(cursor.get_ref()).expect("extract zip");
        assert!(
            std::str::from_utf8(&extracted)
                .expect("utf8")
                .contains("Berlin")
        );
    }
}
