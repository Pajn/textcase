use std::collections::BTreeSet;

use quick_xml::{Reader, events::Event};
use serde_json::Value;

use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let trimmed = bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace());
    if trimmed == Some(b'{') {
        return parse_json(bytes);
    }

    parse_xml(bytes)
}

fn parse_json(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let name = value
        .get("name")
        .and_then(Value::as_object)
        .ok_or("ORCID JSON payload is missing a name object")?;
    let given_name = name
        .get("given-names")
        .and_then(Value::as_object)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_str)
        .ok_or("ORCID JSON payload is missing given-names")?;
    let family_name = name
        .get("family-name")
        .and_then(Value::as_object)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_str)
        .ok_or("ORCID JSON payload is missing family-name")?;

    let canonical = format!("{given_name} {family_name}");
    let mut aliases = BTreeSet::new();
    if let Some(other_names) = value
        .get("other-names")
        .and_then(Value::as_object)
        .and_then(|value| value.get("other-name"))
        .and_then(Value::as_array)
    {
        for other_name in other_names {
            if let Some(alias) = other_name.get("content").and_then(Value::as_str) {
                aliases.insert(alias.to_string());
            }
        }
    }
    aliases.remove(&canonical);

    Ok(vec![SourceRecord {
        canonical,
        aliases: aliases.into_iter().collect(),
        score: 1.5,
    }])
}

fn parse_xml(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let xml = std::str::from_utf8(bytes)?;
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current = Vec::new();
    let mut inside_other_name = false;
    let mut given_name: Option<String> = None;
    let mut family_name: Option<String> = None;
    let mut aliases = BTreeSet::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                current = local_name(event.name().as_ref()).to_vec();
                if current.as_slice() == b"other-name" {
                    inside_other_name = true;
                }
            }
            Event::End(event) => {
                if local_name(event.name().as_ref()) == b"other-name" {
                    inside_other_name = false;
                }
                current.clear();
            }
            Event::Text(event) => {
                let text = event.unescape()?.into_owned();
                match current.as_slice() {
                    b"given-names" => given_name = Some(text),
                    b"family-name" => family_name = Some(text),
                    b"content" if inside_other_name => {
                        aliases.insert(text);
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    let given_name = given_name.or_else(|| extract_first_tag_value(xml, "given-names"));
    let family_name = family_name.or_else(|| extract_first_tag_value(xml, "family-name"));
    if aliases.is_empty() {
        aliases.extend(extract_tag_values(xml, "content"));
    }

    let canonical = match (given_name, family_name) {
        (Some(given), Some(family)) if !given.is_empty() && !family.is_empty() => {
            format!("{given} {family}")
        }
        _ => return Err("ORCID payload did not contain a public name".into()),
    };
    aliases.remove(&canonical);

    Ok(vec![SourceRecord {
        canonical,
        aliases: aliases.into_iter().collect(),
        score: 1.5,
    }])
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn extract_first_tag_value(xml: &str, local_name: &str) -> Option<String> {
    extract_tag_values(xml, local_name).into_iter().next()
}

fn extract_tag_values(xml: &str, tag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let needle = format!(":{tag}");
    let mut search_from = 0;

    while let Some(found) = xml[search_from..].find(&needle) {
        let tag_name_start = search_from + found;
        let Some(open_start) = xml[..tag_name_start].rfind('<') else {
            search_from = tag_name_start + needle.len();
            continue;
        };
        let Some(open_end_rel) = xml[tag_name_start..].find('>') else {
            break;
        };
        let open_end = tag_name_start + open_end_rel;
        let start_tag = &xml[open_start + 1..open_end];
        let tag_name = start_tag.split_whitespace().next().unwrap_or("");
        if tag_name.rsplit(':').next().unwrap_or(tag_name) != tag {
            search_from = open_end + 1;
            continue;
        }

        let mut close_search = open_end + 1;
        while let Some(close_rel) = xml[close_search..].find("</") {
            let close_start = close_search + close_rel;
            let Some(close_end_rel) = xml[close_start..].find('>') else {
                break;
            };
            let close_end = close_start + close_end_rel;
            let close_tag_name = &xml[close_start + 2..close_end];
            if close_tag_name.rsplit(':').next().unwrap_or(close_tag_name) == tag {
                let value = xml[open_end + 1..close_start].trim();
                if !value.is_empty() {
                    values.push(value.to_string());
                }
                search_from = close_end + 1;
                break;
            }
            close_search = close_end + 1;
        }
    }

    values
}

pub fn sample(locale: Option<&str>) -> Vec<u8> {
    let locale = locale.unwrap_or("en");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<personal-details:personal-details xmlns:common="http://www.orcid.org/ns/common" xmlns:personal-details="http://www.orcid.org/ns/personal-details" xmlns:other-name="http://www.orcid.org/ns/other-name">
  <personal-details:name>
    <personal-details:given-names>Ada</personal-details:given-names>
    <personal-details:family-name>Lovelace</personal-details:family-name>
  </personal-details:name>
  <other-name:other-names>
    <other-name:other-name>
      <other-name:content xml:lang="{locale}">Augusta Ada Lovelace</other-name:content>
    </other-name:other-name>
  </other-name:other-names>
</personal-details:personal-details>"#
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_orcid_personal_details_xml() {
        let records = parse(&sample(Some("en"))).expect("parse sample");
        assert_eq!(records[0].canonical, "Ada Lovelace");
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Augusta Ada Lovelace")
        );
    }
}
