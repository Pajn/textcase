use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
};

use flate2::read::GzDecoder;
use quick_xml::{Reader, events::Event};
use zip::ZipArchive;

use super::SourceRecord;

/// Parses a GLEIF LEI-CDF XML payload (golden copy or concatenated file).
///
/// Each `LEIRecord` yields one record: the entity's `LegalName` plus its
/// other and transliterated names as aliases. Previous legal names are
/// skipped — mapping an old name to the current one would rewrite text, not
/// recase it.
pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut records = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    let mut in_record = false;
    let mut skip_name = false;
    let mut canonical: Option<String> = None;
    let mut aliases: BTreeSet<String> = BTreeSet::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                current = event.name().local_name().as_ref().to_vec();
                match current.as_slice() {
                    b"LEIRecord" => {
                        in_record = true;
                        canonical = None;
                        aliases.clear();
                    }
                    b"OtherEntityName" => {
                        skip_name = event.try_get_attribute("type")?.is_some_and(|attribute| {
                            attribute.value.as_ref() == b"PREVIOUS_LEGAL_NAME"
                        });
                    }
                    _ => {}
                }
            }
            Event::End(event) => {
                if event.name().local_name().as_ref() == b"LEIRecord" {
                    in_record = false;
                    if let Some(name) = canonical.take() {
                        aliases.remove(&name);
                        records.push(SourceRecord {
                            score: if super::is_mixed_case(&name) {
                                2.0
                            } else {
                                1.4
                            },
                            canonical: name,
                            aliases: std::mem::take(&mut aliases).into_iter().collect(),
                        });
                    }
                }
                current.clear();
            }
            Event::Text(event) => {
                if !in_record {
                    continue;
                }
                let text = event.unescape()?.trim().to_string();
                if text.is_empty() {
                    continue;
                }
                match current.as_slice() {
                    b"LegalName" if canonical.is_none() => canonical = Some(text),
                    b"OtherEntityName" if !skip_name => {
                        aliases.insert(text);
                    }
                    b"TransliteratedOtherEntityName" => {
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

    if records.is_empty() {
        return Err("GLEIF payload did not yield any records".into());
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    parse(bytes)?;
    Ok(())
}

/// GLEIF distributes the files zipped (golden copy, concatenated file) or
/// gzipped (delta files); a plain XML payload passes through.
pub fn extract_payload(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if bytes.starts_with(b"PK") {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;
            if !file.name().ends_with(".xml") {
                continue;
            }
            let mut out = Vec::new();
            file.read_to_end(&mut out)?;
            return Ok(out);
        }
        return Err("GLEIF archive did not contain an .xml payload".into());
    }
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        GzDecoder::new(bytes).read_to_end(&mut out)?;
        return Ok(out);
    }
    Ok(bytes.to_vec())
}

pub fn sample(_locale: Option<&str>) -> Vec<u8> {
    br#"<lei:LEIData xmlns:lei="http://www.gleif.org/data/schema/leidata/2016">
  <lei:LEIRecords>
    <lei:LEIRecord>
      <lei:LEI>HWUPKR0MPOU8FGXBT394</lei:LEI>
      <lei:Entity>
        <lei:LegalName xml:lang="en">Apple Inc.</lei:LegalName>
        <lei:OtherEntityNames>
          <lei:OtherEntityName type="PREVIOUS_LEGAL_NAME">Apple Computer, Inc.</lei:OtherEntityName>
        </lei:OtherEntityNames>
      </lei:Entity>
    </lei:LEIRecord>
    <lei:LEIRecord>
      <lei:LEI>529900T8BM49AURSDO55</lei:LEI>
      <lei:Entity>
        <lei:LegalName xml:lang="de">Volkswagen Aktiengesellschaft</lei:LegalName>
        <lei:OtherEntityNames>
          <lei:OtherEntityName type="TRADING_OR_OPERATING_NAME">Volkswagen AG</lei:OtherEntityName>
        </lei:OtherEntityNames>
      </lei:Entity>
    </lei:LEIRecord>
  </lei:LEIRecords>
</lei:LEIData>
"#
    .to_vec()
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_lei_records_and_skips_previous_names() {
        let records = parse(&sample(None)).expect("parse sample");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Apple Inc.");
        assert!(records[0].aliases.is_empty());
        assert_eq!(records[1].canonical, "Volkswagen Aktiengesellschaft");
        assert!(
            records[1]
                .aliases
                .iter()
                .any(|alias| alias == "Volkswagen AG")
        );
    }
}
