use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Cursor},
};

use flate2::read::GzDecoder;
use serde_json::Value;

use super::SourceRecord;

pub fn built_in_download(lang: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let normalized = lang.to_ascii_lowercase();
    let url = match normalized.as_str() {
        // Raw wiktextract feeds for Latin-script editions are the most stable built-in option.
        "cs" | "de" | "es" | "fr" | "it" | "nl" | "pl" | "pt" | "tr" => format!(
            "https://kaikki.org/dictionary/downloads/{0}/{0}-extract.jsonl.gz",
            normalized
        ),
        // English currently needs the language-specific words feed because the raw extract is the
        // full multilingual enwiktionary dump.
        "en" => "https://kaikki.org/dictionary/English/words/kaikki.org-dictionary-English-words.jsonl.gz".to_string(),
        _ => {
            return Err(format!(
                "wiktionary built-in fetch does not support --lang {lang}; see docs/sources.md#wiktionary for supported editions and URL guidance"
            )
            .into())
        }
    };

    Ok((url, format!("kaikki-{normalized}")))
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut records = parse_impl(bytes, None, Some(32))?;
    if records.is_empty() {
        return Err("Wiktionary payload did not yield any lexical records".into());
    }
    records.clear();
    Ok(())
}

pub fn parse(
    bytes: &[u8],
    lang: Option<&str>,
) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_impl(bytes, lang, None)
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("de");
    [
        serde_json::json!({
            "word": "Hallo",
            "pos": "noun",
            "lang_code": locale,
            "forms": [
                { "form": "Hallos", "tags": ["plural"] },
                { "form": "de-ndecl", "tags": ["inflection-template"] }
            ]
        }),
        serde_json::json!({
            "word": "laufen",
            "pos": "verb",
            "lang_code": locale,
            "forms": [
                { "form": "läuft", "tags": ["third-person", "singular", "present"] },
                { "form": "lief", "tags": ["past"] }
            ]
        }),
        serde_json::json!({
            "word": "ordo",
            "pos": "noun",
            "lang_code": "la",
            "forms": [{ "form": "ordines", "tags": ["plural"] }]
        }),
    ]
    .iter()
    .map(serde_json::to_string)
    .collect::<Result<Vec<_>, _>>()
    .expect("sample serialization should succeed")
    .join("\n")
    .into_bytes()
}

fn parse_impl(
    bytes: &[u8],
    lang: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let mut reader = line_reader(bytes);
    let target_lang = lang.map(|value| value.to_ascii_lowercase());
    let mut line = String::new();
    let mut records = BTreeMap::<String, f32>::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(trimmed)?;
        if let Some(target_lang) = target_lang.as_deref()
            && value
                .get("lang_code")
                .and_then(Value::as_str)
                .map(|code| !code.eq_ignore_ascii_case(target_lang))
                .unwrap_or(true)
        {
            continue;
        }

        let surfaces = collect_surface_forms(&value);
        if surfaces.is_empty() {
            continue;
        }

        let base_score = score_for_entry(&value);
        for (index, surface) in surfaces.into_iter().enumerate() {
            let score = if index == 0 {
                base_score + 0.15
            } else {
                base_score
            };
            records
                .entry(surface)
                .and_modify(|existing| *existing = existing.max(score))
                .or_insert(score);
        }

        if let Some(limit) = limit
            && records.len() >= limit
        {
            break;
        }
    }

    if records.is_empty() {
        return Err("Wiktionary payload did not yield any lexical records".into());
    }

    Ok(records
        .into_iter()
        .map(|(canonical, score)| SourceRecord {
            canonical,
            aliases: Vec::new(),
            score,
        })
        .collect())
}

fn line_reader(bytes: &[u8]) -> Box<dyn BufRead + '_> {
    if is_gzip(bytes) {
        Box::new(BufReader::new(GzDecoder::new(Cursor::new(bytes))))
    } else {
        Box::new(BufReader::new(Cursor::new(bytes)))
    }
}

fn is_gzip(bytes: &[u8]) -> bool {
    matches!(bytes, [0x1f, 0x8b, ..])
}

fn collect_surface_forms(value: &Value) -> Vec<String> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };

    let mut out = BTreeMap::<String, ()>::new();
    if let Some(word) = object.get("word").and_then(Value::as_str)
        && is_usable_surface(word)
    {
        out.insert(word.to_string(), ());
    }

    if let Some(forms) = object.get("forms").and_then(Value::as_array) {
        for form in forms {
            let Some(surface) = form.get("form").and_then(Value::as_str) else {
                continue;
            };
            if form_has_excluded_tags(form) || !is_usable_surface(surface) {
                continue;
            }
            out.insert(surface.to_string(), ());
        }
    }

    out.into_keys().collect()
}

fn form_has_excluded_tags(form: &Value) -> bool {
    form.get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter().filter_map(Value::as_str).any(|tag| {
                matches!(
                    tag,
                    "inflection-template" | "table-tags" | "class" | "romanization"
                )
            })
        })
        .unwrap_or(false)
}

fn is_usable_surface(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.chars().any(|ch| ch.is_alphanumeric())
        && !trimmed.contains('\t')
        && !trimmed.contains('\n')
}

fn score_for_entry(value: &Value) -> f32 {
    let pos = value
        .get("pos")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let word = value
        .get("word")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let mut score = match pos.as_str() {
        "name" | "proper noun" | "proper-noun" => 2.4,
        "noun" => 2.0,
        "adj" | "adjective" => 1.6,
        "verb" => 1.5,
        "adv" | "adverb" => 1.2,
        _ => 1.0,
    };
    if word.chars().next().is_some_and(char::is_uppercase) {
        score += 0.2;
    }
    score
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{Compression, write::GzEncoder};

    use super::{parse, sample};

    #[test]
    fn parses_jsonl_and_filters_language() {
        let records = parse(&sample(Some("de")), Some("de")).expect("parse sample");
        let forms = records
            .iter()
            .map(|record| record.canonical.as_str())
            .collect::<Vec<_>>();
        assert!(forms.contains(&"Hallo"));
        assert!(forms.contains(&"Hallos"));
        assert!(forms.contains(&"läuft"));
        assert!(!forms.contains(&"ordo"));
    }

    #[test]
    fn parses_gzipped_jsonl() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&sample(Some("de")))
            .expect("write sample");
        let bytes = encoder.finish().expect("finish gzip");
        let records = parse(&bytes, Some("de")).expect("parse sample");
        assert!(records.iter().any(|record| record.canonical == "Hallo"));
    }
}
