mod dbpedia;
mod geonames;
mod getty;
mod gnd;
mod musicbrainz;
mod omw;
mod openstreetmap;
mod orcid;
mod ud_german_gsd;
mod wikidata;
mod wiktionary;

use std::collections::BTreeMap;
use std::fmt;

use clap::ValueEnum;
use serde_json::Value;
use textcase::lexicon::{PreparedKind, PreparedLexicon, PreparedPayload};
use textcase::plugin::{LicenseMetadata, SourceMetadata};

use crate::{
    manifest::FetchedSourceManifest,
    prepare::{canonical_map, multiword_map, ranked},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum SourceId {
    Wikidata,
    Geonames,
    UdGermanGsd,
    Gnd,
    Orcid,
    Musicbrainz,
    Getty,
    Wiktionary,
    Dbpedia,
    Openstreetmap,
    Omw,
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SourceId::Wikidata => "wikidata",
            SourceId::Geonames => "geonames",
            SourceId::UdGermanGsd => "ud-german-gsd",
            SourceId::Gnd => "gnd",
            SourceId::Orcid => "orcid",
            SourceId::Musicbrainz => "musicbrainz",
            SourceId::Getty => "getty",
            SourceId::Wiktionary => "wiktionary",
            SourceId::Dbpedia => "dbpedia",
            SourceId::Openstreetmap => "openstreetmap",
            SourceId::Omw => "omw",
        };
        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceClass {
    Green,
    Yellow,
    Orange,
    Gray,
}

impl fmt::Display for SourceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SourceClass::Green => "green",
            SourceClass::Yellow => "yellow",
            SourceClass::Orange => "orange",
            SourceClass::Gray => "gray",
        };
        write!(f, "{value}")
    }
}

#[derive(Clone, Debug)]
pub struct SourceDescriptor {
    pub id: SourceId,
    pub display_name: &'static str,
    pub class: SourceClass,
    pub license_name: &'static str,
    pub license_summary: &'static str,
    pub acknowledgement_flag: Option<&'static str>,
    pub recommended: bool,
    pub plugin_kinds: &'static [PreparedKind],
    pub domain_tags: &'static [&'static str],
    pub docs_anchor: &'static str,
    pub purpose: &'static str,
    pub bundling_policy: &'static str,
}

#[derive(Clone, Debug)]
pub struct SourceRecord {
    pub canonical: String,
    pub aliases: Vec<String>,
    pub score: f32,
}

pub struct FetchPlan {
    pub urls: Vec<String>,
    pub source_url: String,
    pub version: String,
    pub output_suffix: String,
}

const WIKIDATA_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const GEONAMES_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const UD_KINDS: &[PreparedKind] = &[PreparedKind::RankedCandidates];
const GND_KINDS: &[PreparedKind] = &[
    PreparedKind::CanonicalMap,
    PreparedKind::MultiwordMap,
    PreparedKind::ProtectedForms,
];
const ORCID_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const MUSICBRAINZ_KINDS: &[PreparedKind] = &[
    PreparedKind::CanonicalMap,
    PreparedKind::MultiwordMap,
    PreparedKind::ProtectedForms,
];
const GETTY_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const WIKTIONARY_KINDS: &[PreparedKind] = &[PreparedKind::WordSet, PreparedKind::RankedCandidates];
const DBPEDIA_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const OPENSTREETMAP_KINDS: &[PreparedKind] =
    &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const OMW_KINDS: &[PreparedKind] = &[PreparedKind::WordSet, PreparedKind::RankedCandidates];

const DESCRIPTORS: &[SourceDescriptor] = &[
    SourceDescriptor {
        id: SourceId::Wikidata,
        display_name: "Wikidata",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "CC0 multilingual entity labels and aliases",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: WIKIDATA_KINDS,
        domain_tags: &["proper-nouns", "multilingual", "entities"],
        docs_anchor: "#wikidata",
        purpose: "multilingual proper nouns and aliases",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Geonames,
        display_name: "GeoNames",
        class: SourceClass::Yellow,
        license_name: "GeoNames attribution",
        license_summary: "geographic names with attribution obligations",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: GEONAMES_KINDS,
        domain_tags: &["geography", "places"],
        docs_anchor: "#geonames",
        purpose: "geographical names and alternates",
        bundling_policy: "external plugin only with attribution guidance",
    },
    SourceDescriptor {
        id: SourceId::UdGermanGsd,
        display_name: "UD German GSD",
        class: SourceClass::Orange,
        license_name: "CC BY-SA 4.0",
        license_summary: "optional German ranking hints with share-alike obligations",
        acknowledgement_flag: Some("--acknowledge-cc-by-sa"),
        recommended: false,
        plugin_kinds: UD_KINDS,
        domain_tags: &["german", "ranking", "nlp"],
        docs_anchor: "#ud-german-gsd",
        purpose: "German ranked candidates",
        bundling_policy: "optional end-user plugin only",
    },
    SourceDescriptor {
        id: SourceId::Gnd,
        display_name: "GND",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "authority names for persons, places, and works",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: GND_KINDS,
        domain_tags: &["authority", "names", "german"],
        docs_anchor: "#gnd",
        purpose: "authority-style proper nouns",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Orcid,
        display_name: "ORCID",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "researcher and affiliation names",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: ORCID_KINDS,
        domain_tags: &["people", "research"],
        docs_anchor: "#orcid",
        purpose: "researcher and affiliation names",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Musicbrainz,
        display_name: "MusicBrainz",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "artists, bands, albums, and works",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: MUSICBRAINZ_KINDS,
        domain_tags: &["music", "media"],
        docs_anchor: "#musicbrainz",
        purpose: "music and media proper nouns",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Getty,
        display_name: "Getty",
        class: SourceClass::Yellow,
        license_name: "ODC-By 1.0",
        license_summary: "cultural and heritage vocabulary with attribution",
        acknowledgement_flag: None,
        recommended: false,
        plugin_kinds: GETTY_KINDS,
        domain_tags: &["arts", "culture"],
        docs_anchor: "#getty",
        purpose: "culture and arts authority names",
        bundling_policy: "external plugin only with attribution guidance",
    },
    SourceDescriptor {
        id: SourceId::Wiktionary,
        display_name: "Wiktionary",
        class: SourceClass::Orange,
        license_name: "CC BY-SA / GFDL",
        license_summary: "rich lexical hints with share-alike obligations",
        acknowledgement_flag: Some("--acknowledge-share-alike"),
        recommended: false,
        plugin_kinds: WIKTIONARY_KINDS,
        domain_tags: &["lexical", "hints"],
        docs_anchor: "#wiktionary",
        purpose: "lexical hints and candidate forms",
        bundling_policy: "optional end-user plugin only",
    },
    SourceDescriptor {
        id: SourceId::Dbpedia,
        display_name: "DBpedia",
        class: SourceClass::Orange,
        license_name: "share-alike derived data",
        license_summary: "broad entity graph with stronger obligations",
        acknowledgement_flag: Some("--acknowledge-share-alike"),
        recommended: false,
        plugin_kinds: DBPEDIA_KINDS,
        domain_tags: &["entities", "multilingual"],
        docs_anchor: "#dbpedia",
        purpose: "entity names and multilingual labels",
        bundling_policy: "optional end-user plugin only",
    },
    SourceDescriptor {
        id: SourceId::Openstreetmap,
        display_name: "OpenStreetMap",
        class: SourceClass::Orange,
        license_name: "ODbL",
        license_summary: "fine-grained locality names with ODbL obligations",
        acknowledgement_flag: Some("--acknowledge-odbl"),
        recommended: false,
        plugin_kinds: OPENSTREETMAP_KINDS,
        domain_tags: &["geography", "locality"],
        docs_anchor: "#openstreetmap",
        purpose: "local places, roads, and locality names",
        bundling_policy: "optional end-user plugin only",
    },
    SourceDescriptor {
        id: SourceId::Omw,
        display_name: "Open Multilingual Wordnet",
        class: SourceClass::Gray,
        license_name: "open / mixed practical value",
        license_summary: "experimental lexical hints source",
        acknowledgement_flag: None,
        recommended: false,
        plugin_kinds: OMW_KINDS,
        domain_tags: &["lexical", "experimental"],
        docs_anchor: "#omw",
        purpose: "lexical hints and semantic groupings",
        bundling_policy: "experimental external plugin only",
    },
];

pub fn descriptors() -> &'static [SourceDescriptor] {
    DESCRIPTORS
}

pub fn descriptor(id: SourceId) -> &'static SourceDescriptor {
    DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.id == id)
        .expect("source descriptor exists")
}

pub fn require_acknowledgement(
    source: SourceId,
    acknowledge_cc_by_sa: bool,
    acknowledge_share_alike: bool,
    acknowledge_odbl: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = descriptor(source);
    match descriptor.acknowledgement_flag {
        Some("--acknowledge-cc-by-sa") if !acknowledge_cc_by_sa => {
            Err("missing required --acknowledge-cc-by-sa flag".into())
        }
        Some("--acknowledge-share-alike") if !acknowledge_share_alike => {
            Err("missing required --acknowledge-share-alike flag".into())
        }
        Some("--acknowledge-odbl") if !acknowledge_odbl => {
            Err("missing required --acknowledge-odbl flag".into())
        }
        _ => Ok(()),
    }
}

pub fn suggested_output_name(source: SourceId, suffix: &str) -> String {
    let extension = match source {
        SourceId::Geonames => "tsv",
        SourceId::UdGermanGsd => "conllu",
        _ => "json",
    };
    format!("{}-{}.{}", source, suffix.to_lowercase(), extension)
}

pub fn built_in_fetch_plan(
    source: SourceId,
    _lang: Option<&str>,
    country: Option<&str>,
    _region: Option<&str>,
) -> Result<FetchPlan, Box<dyn std::error::Error>> {
    match source {
        SourceId::Geonames => {
            let (country_code, output_suffix) = match country {
                Some(code) => (code.to_ascii_uppercase(), code.to_ascii_lowercase()),
                None => ("allCountries".to_string(), "allcountries".to_string()),
            };
            Ok(FetchPlan {
                source_url: format!(
                    "https://download.geonames.org/export/dump/{country_code}.zip"
                ),
                urls: vec![format!(
                    "https://download.geonames.org/export/dump/{country_code}.zip"
                )],
                version: country_code.clone(),
                output_suffix,
            })
        }
        SourceId::UdGermanGsd => Ok(FetchPlan {
            source_url:
                "https://raw.githubusercontent.com/UniversalDependencies/UD_German-GSD/r2.13/"
                    .to_string(),
            urls: vec![
                "https://raw.githubusercontent.com/UniversalDependencies/UD_German-GSD/r2.13/de_gsd-ud-train.conllu".to_string(),
                "https://raw.githubusercontent.com/UniversalDependencies/UD_German-GSD/r2.13/de_gsd-ud-dev.conllu".to_string(),
                "https://raw.githubusercontent.com/UniversalDependencies/UD_German-GSD/r2.13/de_gsd-ud-test.conllu".to_string(),
            ],
            version: "r2.13".to_string(),
            output_suffix: "r2.13".to_string(),
        }),
        _ => Err(format!(
            "{source} does not have a built-in fetch workflow yet; pass --url or --sample"
        )
        .into()),
    }
}

pub fn normalize_download(
    source: SourceId,
    downloads: Vec<Vec<u8>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match source {
        SourceId::Geonames => {
            let archive = downloads
                .into_iter()
                .next()
                .ok_or("GeoNames fetch returned no payload")?;
            geonames::extract_zip(&archive)
        }
        SourceId::UdGermanGsd => {
            Ok(downloads
                .into_iter()
                .enumerate()
                .fold(Vec::new(), |mut merged, (index, bytes)| {
                    if index > 0 && !merged.ends_with(b"\n") {
                        merged.push(b'\n');
                    }
                    merged.extend_from_slice(&bytes);
                    if !merged.ends_with(b"\n") {
                        merged.push(b'\n');
                    }
                    merged
                }))
        }
        _ => Ok(downloads
            .into_iter()
            .next()
            .ok_or("download returned no payload")?),
    }
}

pub fn validate_source_bytes(
    source: SourceId,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    match source {
        SourceId::Geonames => geonames::validate(bytes),
        SourceId::UdGermanGsd => ud_german_gsd::validate(bytes),
        SourceId::Wikidata => {
            wikidata::parse(bytes, None)?;
            Ok(())
        }
        SourceId::Gnd => {
            gnd::parse(bytes)?;
            Ok(())
        }
        SourceId::Orcid => {
            orcid::parse(bytes)?;
            Ok(())
        }
        SourceId::Musicbrainz => {
            musicbrainz::parse(bytes)?;
            Ok(())
        }
        SourceId::Getty => {
            getty::parse(bytes)?;
            Ok(())
        }
        SourceId::Dbpedia => {
            dbpedia::parse(bytes)?;
            Ok(())
        }
        SourceId::Openstreetmap => {
            openstreetmap::parse(bytes)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn sample_payload(
    source: SourceId,
    lang: Option<&str>,
    country: Option<&str>,
    region: Option<&str>,
) -> Vec<u8> {
    match source {
        SourceId::Wikidata => wikidata::sample(lang),
        SourceId::Geonames => geonames::sample(country),
        SourceId::UdGermanGsd => ud_german_gsd::sample(),
        SourceId::Gnd => gnd::sample(lang),
        SourceId::Orcid => orcid::sample(lang),
        SourceId::Musicbrainz => musicbrainz::sample(lang),
        SourceId::Getty => getty::sample(lang),
        SourceId::Wiktionary => wiktionary::sample(lang),
        SourceId::Dbpedia => dbpedia::sample(lang),
        SourceId::Openstreetmap => openstreetmap::sample(region),
        SourceId::Omw => omw::sample(lang),
    }
}

pub fn prepare_source(
    source: SourceId,
    bytes: &[u8],
    kind: PreparedKind,
    lang: Option<&str>,
    provenance: Option<&FetchedSourceManifest>,
) -> Result<PreparedLexicon, Box<dyn std::error::Error>> {
    let descriptor = descriptor(source);
    if !descriptor.plugin_kinds.contains(&kind) {
        return Err(format!("{source} does not support {:?}", kind).into());
    }

    let records = match source {
        SourceId::Wikidata => wikidata::parse(bytes, lang)?,
        SourceId::Geonames => geonames::parse(bytes)?,
        SourceId::UdGermanGsd => ud_german_gsd::parse(bytes)?,
        SourceId::Gnd => gnd::parse(bytes)?,
        SourceId::Orcid => orcid::parse(bytes)?,
        SourceId::Musicbrainz => musicbrainz::parse(bytes)?,
        SourceId::Getty => getty::parse(bytes)?,
        SourceId::Wiktionary => wiktionary::parse(bytes)?,
        SourceId::Dbpedia => dbpedia::parse(bytes)?,
        SourceId::Openstreetmap => openstreetmap::parse(bytes)?,
        SourceId::Omw => omw::parse(bytes)?,
    };

    let payload = match kind {
        PreparedKind::CanonicalMap => PreparedPayload::CanonicalMap(canonical_map::build(&records)),
        PreparedKind::MultiwordMap => PreparedPayload::MultiwordMap(multiword_map::build(&records)),
        PreparedKind::ProtectedForms => {
            PreparedPayload::ProtectedForms(canonical_map::build_protected(&records))
        }
        PreparedKind::WordSet => PreparedPayload::WordSet(build_word_set(&records)),
        PreparedKind::RankedCandidates => {
            PreparedPayload::RankedCandidates(ranked::build(&records))
        }
    };

    Ok(PreparedLexicon {
        name: format!("{}-{}", source, kind_label(kind)),
        kind,
        locale: lang.unwrap_or("und").to_string(),
        license: LicenseMetadata {
            name: descriptor.license_name.to_string(),
            summary: descriptor.license_summary.to_string(),
            acknowledgement_flag: descriptor.acknowledgement_flag.map(str::to_owned),
        },
        sources: vec![SourceMetadata {
            id: source.to_string(),
            display_name: descriptor.display_name.to_string(),
            url: provenance
                .map(|manifest| manifest.source_url.clone())
                .unwrap_or_else(|| format!("https://docs.invalid/textcase/sources/{}", source)),
            version: provenance
                .map(|manifest| manifest.version.clone())
                .unwrap_or_else(|| "user-supplied".to_string()),
            class: descriptor.class.to_string(),
        }],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload,
    })
}

fn build_word_set(records: &[SourceRecord]) -> Vec<String> {
    let mut words = BTreeMap::new();
    for record in records {
        words.insert(record.canonical.to_lowercase(), ());
        for alias in &record.aliases {
            words.insert(alias.to_lowercase(), ());
        }
    }
    words.into_keys().collect()
}

fn kind_label(kind: PreparedKind) -> &'static str {
    match kind {
        PreparedKind::WordSet => "word-set",
        PreparedKind::CanonicalMap => "canonical-map",
        PreparedKind::MultiwordMap => "multiword-map",
        PreparedKind::RankedCandidates => "ranked-candidates",
        PreparedKind::ProtectedForms => "protected-forms",
    }
}

pub(crate) fn parse_json_records(
    bytes: &[u8],
) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(bytes)?;
    let items = match value {
        Value::Array(items) => items,
        Value::Object(mut object) => object
            .remove("items")
            .or_else(|| object.remove("records"))
            .or_else(|| object.remove("entities"))
            .map(|nested| match nested {
                Value::Array(items) => items,
                other => vec![other],
            })
            .unwrap_or_else(|| vec![Value::Object(object)]),
        other => vec![other],
    };

    let mut records = Vec::new();
    for item in items {
        if let Some(record) = record_from_value(&item) {
            records.push(record);
        }
    }
    Ok(records)
}

pub(crate) fn sample_json(id: &str, locale: Option<&str>, rows: &[(&str, &[&str])]) -> Vec<u8> {
    let payload = serde_json::json!({
        "source": id,
        "locale": locale.unwrap_or("und"),
        "items": rows.iter().map(|(canonical, aliases)| serde_json::json!({
            "canonical": canonical,
            "aliases": aliases,
            "score": if canonical.contains(' ') { 1.5 } else { 1.0 },
        })).collect::<Vec<_>>(),
    });
    serde_json::to_vec_pretty(&payload).expect("sample serialization should succeed")
}

fn record_from_value(value: &Value) -> Option<SourceRecord> {
    let object = value.as_object()?;
    let canonical = string_fields(
        object,
        &[
            "canonical",
            "name",
            "label",
            "title",
            "display_name",
            "value",
        ],
    )?;
    let mut aliases = Vec::new();
    for key in [
        "aliases",
        "alias",
        "aka",
        "labels",
        "names",
        "alt_name",
        "alternate_names",
    ] {
        if let Some(value) = object.get(key) {
            collect_aliases(value, &mut aliases);
        }
    }
    let score = object
        .get("score")
        .and_then(Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(1.0);
    Some(SourceRecord {
        canonical,
        aliases,
        score,
    })
}

fn string_fields(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| object.get(*key))
        .and_then(value_to_string)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.trim().to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Array(values) => values.iter().find_map(value_to_string),
        Value::Object(map) => map.values().find_map(value_to_string),
        _ => None,
    }
}

fn collect_aliases(value: &Value, aliases: &mut Vec<String>) {
    match value {
        Value::String(value) => split_aliases(value, aliases),
        Value::Array(values) => {
            for value in values {
                collect_aliases(value, aliases);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_aliases(value, aliases);
            }
        }
        _ => {}
    }
}

fn split_aliases(value: &str, aliases: &mut Vec<String>) {
    for part in value.split(['|', ';']) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            aliases.push(trimmed.to_string());
        }
    }
}
