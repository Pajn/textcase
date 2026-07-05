use std::{
    collections::BTreeSet,
    io::{BufRead, BufReader},
};

use flate2::read::GzDecoder;
use quick_xml::{Reader, events::Event};

use super::SourceRecord;

/// Parses a Discogs monthly data dump (artists, labels, masters, or releases).
///
/// Records are only created for top-level entries: names under `members`,
/// `groups`, or `sublabels` belong to entities that appear as their own
/// top-level records elsewhere in the dump.
///
/// The payload is streamed through `quick_xml`: a gzipped dump (as distributed)
/// is decoded on the fly and a plain XML payload is read as-is, so the
/// decompressed multi-gigabyte XML is never fully held in memory.
pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_reader(xml_reader(bytes))
}

/// Wraps the payload in a streaming reader, transparently gunzipping a gzip
/// magic-numbered dump and passing plain XML through unchanged.
fn xml_reader(bytes: &[u8]) -> Box<dyn BufRead + '_> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        Box::new(BufReader::new(GzDecoder::new(bytes)))
    } else {
        Box::new(BufReader::new(bytes))
    }
}

fn parse_reader<R: BufRead>(reader: R) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut path: Vec<Vec<u8>> = Vec::new();
    let mut records = Vec::new();
    let mut canonical: Option<String> = None;
    let mut aliases: BTreeSet<String> = BTreeSet::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                path.push(event.local_name().as_ref().to_vec());
                if is_record_element(&path) {
                    canonical = None;
                    aliases.clear();
                }
            }
            Event::End(_) => {
                if is_record_element(&path)
                    && let Some(name) = canonical.take()
                {
                    aliases.remove(&name);
                    records.push(SourceRecord {
                        score: if super::is_mixed_case(&name) {
                            2.0
                        } else {
                            1.2
                        },
                        canonical: name,
                        aliases: std::mem::take(&mut aliases).into_iter().collect(),
                    });
                }
                path.pop();
            }
            Event::Text(event) => {
                // Skip irrelevant text nodes (ids, profiles, member names) before
                // paying for unescaping and allocation.
                let role = name_role(&path);
                if matches!(role, NameRole::None) {
                    continue;
                }
                let text = event.unescape()?.into_owned();
                let text = strip_disambiguation(text.trim());
                if text.is_empty() {
                    continue;
                }
                match role {
                    NameRole::Canonical if canonical.is_none() => canonical = Some(text),
                    NameRole::Canonical | NameRole::Alias => {
                        aliases.insert(text);
                    }
                    NameRole::None => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    if records.is_empty() {
        return Err("Discogs payload did not yield any records".into());
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    parse(bytes)?;
    Ok(())
}

pub fn sample(_locale: Option<&str>) -> Vec<u8> {
    br#"<artists>
  <artist>
    <id>1289</id>
    <name>Daft Punk</name>
    <namevariations><name>Daftpunk</name></namevariations>
    <aliases><name id="2933">The Third Twin</name></aliases>
    <members><name id="1590">Thomas Bangalter</name></members>
  </artist>
  <artist>
    <id>5</id>
    <name>Aphex Twin (2)</name>
    <realname>Richard David James</realname>
  </artist>
</artists>
"#
    .to_vec()
}

enum NameRole {
    Canonical,
    Alias,
    None,
}

/// A record is a direct child of the dump root: `artists/artist`,
/// `labels/label`, `masters/master`, or `releases/release`.
fn is_record_element(path: &[Vec<u8>]) -> bool {
    path.len() == 2
        && matches!(
            path[1].as_slice(),
            b"artist" | b"label" | b"master" | b"release"
        )
}

fn name_role(path: &[Vec<u8>]) -> NameRole {
    // <name> or <title> directly under the record element.
    if path.len() == 3 && is_record_element(&path[..2]) {
        return match path[2].as_slice() {
            b"name" | b"title" => NameRole::Canonical,
            b"realname" => NameRole::Alias,
            _ => NameRole::None,
        };
    }
    // <name> inside the record's namevariations or aliases containers.
    if path.len() == 4
        && is_record_element(&path[..2])
        && matches!(path[2].as_slice(), b"namevariations" | b"aliases")
        && path[3].as_slice() == b"name"
    {
        return NameRole::Alias;
    }
    NameRole::None
}

/// Removes the Discogs disambiguation suffix: "Aphex Twin (2)" → "Aphex Twin".
fn strip_disambiguation(name: &str) -> String {
    if let Some(start) = name.rfind(" (")
        && name.ends_with(')')
    {
        let digits = &name[start + 2..name.len() - 1];
        if !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()) {
            return name[..start].to_string();
        }
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{Compression, write::GzEncoder};

    use super::{parse, sample};

    #[test]
    fn parses_artist_dump_entries() {
        let records = parse(&sample(None)).expect("parse sample");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Daft Punk");
        assert!(records[0].aliases.iter().any(|alias| alias == "Daftpunk"));
        assert!(
            records[0]
                .aliases
                .iter()
                .any(|alias| alias == "The Third Twin")
        );
        // Member names belong to their own top-level records.
        assert!(
            !records[0]
                .aliases
                .iter()
                .any(|alias| alias == "Thomas Bangalter")
        );
        // The disambiguation suffix is stripped.
        assert_eq!(records[1].canonical, "Aphex Twin");
        assert!(
            records[1]
                .aliases
                .iter()
                .any(|alias| alias == "Richard David James")
        );
    }

    #[test]
    fn parses_label_dump_entries() {
        let xml = br#"<labels>
  <label><id>1</id><name>Planet E</name></label>
</labels>"#;
        let records = parse(xml).expect("parse labels");
        assert_eq!(records[0].canonical, "Planet E");
    }

    #[test]
    fn parses_gzipped_dumps_without_decompressing_first() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&sample(None)).expect("write");
        let compressed = encoder.finish().expect("finish");
        // The gzip is streamed straight into the parser, yielding the same
        // records as the plain payload.
        let records = parse(&compressed).expect("parse gzipped");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Daft Punk");
    }
}
