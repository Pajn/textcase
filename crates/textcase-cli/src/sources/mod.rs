mod cldr;
mod dbpedia;
mod discogs;
mod geonames;
mod getty;
mod gleif;
mod gnd;
mod musicbrainz;
mod natural_earth;
mod openstreetmap;
mod orcid;
mod ror;
mod ud_german_gsd;
mod wikidata;
mod wiktionary;

use std::collections::BTreeMap;
use std::fmt;

use clap::ValueEnum;
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
    Discogs,
    Gleif,
    Ror,
    Cldr,
    NaturalEarth,
    UdGermanGsd,
    Gnd,
    Orcid,
    Musicbrainz,
    Getty,
    Wiktionary,
    Dbpedia,
    Openstreetmap,
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SourceId::Wikidata => "wikidata",
            SourceId::Geonames => "geonames",
            SourceId::Discogs => "discogs",
            SourceId::Gleif => "gleif",
            SourceId::Ror => "ror",
            SourceId::Cldr => "cldr",
            SourceId::NaturalEarth => "natural-earth",
            SourceId::UdGermanGsd => "ud-german-gsd",
            SourceId::Gnd => "gnd",
            SourceId::Orcid => "orcid",
            SourceId::Musicbrainz => "musicbrainz",
            SourceId::Getty => "getty",
            SourceId::Wiktionary => "wiktionary",
            SourceId::Dbpedia => "dbpedia",
            SourceId::Openstreetmap => "openstreetmap",
        };
        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceClass {
    Green,
    Yellow,
    Orange,
}

impl fmt::Display for SourceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SourceClass::Green => "green",
            SourceClass::Yellow => "yellow",
            SourceClass::Orange => "orange",
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

/// Whether `value` mixes letter cases (`iPhone`, `LaTeX`), a signal that a
/// name carries intentional brand casing worth scoring above plain words.
pub(super) fn is_mixed_case(value: &str) -> bool {
    value.chars().any(char::is_lowercase) && value.chars().any(char::is_uppercase)
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
const DISCOGS_KINDS: &[PreparedKind] = &[
    PreparedKind::CanonicalMap,
    PreparedKind::MultiwordMap,
    PreparedKind::ProtectedForms,
];
const GLEIF_KINDS: &[PreparedKind] = &[
    PreparedKind::CanonicalMap,
    PreparedKind::MultiwordMap,
    PreparedKind::ProtectedForms,
];
const NATURAL_EARTH_KINDS: &[PreparedKind] =
    &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const CLDR_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const ROR_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const GETTY_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const WIKTIONARY_KINDS: &[PreparedKind] = &[PreparedKind::WordSet, PreparedKind::RankedCandidates];
const DBPEDIA_KINDS: &[PreparedKind] = &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];
const OPENSTREETMAP_KINDS: &[PreparedKind] =
    &[PreparedKind::CanonicalMap, PreparedKind::MultiwordMap];

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
        id: SourceId::Discogs,
        display_name: "Discogs",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "CC0 monthly dumps of artists, labels, and releases",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: DISCOGS_KINDS,
        domain_tags: &["music", "media"],
        docs_anchor: "#discogs",
        purpose: "music artist, label, and release names",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Gleif,
        display_name: "GLEIF",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "global legal entity names from the LEI system",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: GLEIF_KINDS,
        domain_tags: &["organizations", "companies"],
        docs_anchor: "#gleif",
        purpose: "company and organization legal names",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Ror,
        display_name: "ROR",
        class: SourceClass::Green,
        license_name: "CC0",
        license_summary: "research organization names from the ROR registry",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: ROR_KINDS,
        domain_tags: &["organizations", "research"],
        docs_anchor: "#ror",
        purpose: "research organization and institution names",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::Cldr,
        display_name: "Unicode CLDR",
        class: SourceClass::Green,
        license_name: "Unicode License v3",
        license_summary: "language and territory display names per locale",
        acknowledgement_flag: None,
        recommended: true,
        plugin_kinds: CLDR_KINDS,
        domain_tags: &["locale", "names", "multilingual"],
        docs_anchor: "#cldr",
        purpose: "language, country, and region display names",
        bundling_policy: "external plugin only",
    },
    SourceDescriptor {
        id: SourceId::NaturalEarth,
        display_name: "Natural Earth",
        class: SourceClass::Green,
        license_name: "public domain",
        license_summary: "world-scale country and city names, no attribution required",
        acknowledgement_flag: None,
        recommended: false,
        plugin_kinds: NATURAL_EARTH_KINDS,
        domain_tags: &["geography", "places"],
        docs_anchor: "#natural-earth",
        purpose: "coarse geographic names without attribution obligations",
        bundling_policy: "external plugin only",
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
        SourceId::Wiktionary => "jsonl.gz",
        SourceId::Discogs => "xml.gz",
        SourceId::NaturalEarth => "geojson",
        SourceId::Gleif => "xml",
        _ => "json",
    };
    format!("{}-{}.{}", source, suffix.to_lowercase(), extension)
}

pub fn built_in_fetch_plan(
    source: SourceId,
    lang: Option<&str>,
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
        SourceId::NaturalEarth => {
            let (source_url, urls) = natural_earth::built_in_download();
            Ok(FetchPlan {
                source_url,
                urls,
                version: natural_earth::NATURAL_EARTH_VERSION.to_string(),
                output_suffix: "world".to_string(),
            })
        }
        SourceId::Cldr => {
            let lang = lang.ok_or("cldr built-in fetch requires --lang")?;
            let (source_url, urls) = cldr::built_in_download(lang);
            Ok(FetchPlan {
                source_url,
                urls,
                version: cldr::CLDR_JSON_VERSION.to_string(),
                output_suffix: lang.to_ascii_lowercase(),
            })
        }
        SourceId::Wiktionary => {
            let lang = lang.ok_or("wiktionary built-in fetch requires --lang")?;
            let (url, version) = wiktionary::built_in_download(lang)?;
            Ok(FetchPlan {
                source_url: url.clone(),
                urls: vec![url],
                version,
                output_suffix: lang.to_ascii_lowercase(),
            })
        }
        _ => Err(format!(
            "{source} does not have a built-in fetch workflow; pass --url with a documented upstream endpoint"
        )
        .into()),
    }
}

pub fn fetch_guidance(source: SourceId) -> &'static str {
    match source {
        SourceId::Geonames => {
            "built-in download; use --country for a national extract or omit it for allCountries"
        }
        SourceId::UdGermanGsd => {
            "built-in download; fetches the UD German GSD r2.13 train/dev/test files"
        }
        SourceId::Wiktionary => {
            "built-in download; requires --lang and --acknowledge-share-alike, with Kaikki-backed editions for de/es/fr/it/nl/pl/pt/tr/cs and English words"
        }
        SourceId::Wikidata => {
            "URL-driven; point --url at a Wikidata entity export such as Special:EntityData/Q64.json or a curated entities JSON file"
        }
        SourceId::Gnd => {
            "URL-driven; point --url at a lobid.org GND record or search result JSON feed"
        }
        SourceId::Orcid => {
            "URL-driven; point --url at an ORCID public API personal-details document for a curated researcher set"
        }
        SourceId::Musicbrainz => {
            "URL-driven; point --url at a MusicBrainz ws/2 search result or entity JSON document"
        }
        SourceId::Getty => {
            "URL-driven; point --url at a Getty linked-art JSON record such as AAT or TGN"
        }
        SourceId::Dbpedia => {
            "URL-driven; point --url at a DBpedia Lookup API result or a resource-graph JSON response"
        }
        SourceId::Openstreetmap => {
            "URL-driven; point --url at a Nominatim JSON search scoped to the locality set you need, and acknowledge ODbL"
        }
        SourceId::Gleif => {
            "URL-driven; point --url at a GLEIF golden copy or concatenated LEI file (zip, gz, or xml) from gleif.org/en/lei-data"
        }
        SourceId::NaturalEarth => {
            "built-in download; fetches world countries and populated places GeoJSON from a pinned Natural Earth release"
        }
        SourceId::Cldr => {
            "built-in download; requires --lang and fetches that locale's territory and language display names from cldr-json (pinned release)"
        }
        SourceId::Ror => {
            "URL-driven; point --url at the latest ROR data dump zip from Zenodo (https://doi.org/10.5281/zenodo.6347574)"
        }
        SourceId::Discogs => {
            "URL-driven; point --url at a monthly dump from data.discogs.com, e.g. https://discogs-data-dumps.s3.us-west-2.amazonaws.com/data/2025/discogs_20250601_artists.xml.gz (the URL carries the dump date)"
        }
    }
}

/// Whether the fetched download must be normalized in memory (zip extraction,
/// JSON merge, or multi-file concatenation) before it can be written, versus a
/// single payload that is streamed straight to disk. Mirrors the non-default
/// arms of [`normalize_download`].
pub fn requires_normalization(source: SourceId) -> bool {
    matches!(
        source,
        SourceId::Geonames
            | SourceId::Cldr
            | SourceId::NaturalEarth
            | SourceId::Ror
            | SourceId::Gleif
            | SourceId::UdGermanGsd
    )
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
        SourceId::Cldr => cldr::merge(downloads),
        SourceId::NaturalEarth => natural_earth::merge(downloads),
        SourceId::Ror => {
            let payload = downloads
                .into_iter()
                .next()
                .ok_or("ROR fetch returned no payload")?;
            ror::extract_payload(&payload)
        }
        SourceId::Gleif => {
            let payload = downloads
                .into_iter()
                .next()
                .ok_or("GLEIF fetch returned no payload")?;
            gleif::extract_payload(&payload)
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
        SourceId::Discogs => discogs::validate(bytes),
        SourceId::Gleif => gleif::validate(bytes),
        SourceId::Ror => ror::validate(bytes),
        SourceId::Cldr => cldr::validate(bytes),
        SourceId::NaturalEarth => natural_earth::validate(bytes),
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
        SourceId::Wiktionary => wiktionary::validate(bytes),
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
        SourceId::Discogs => discogs::sample(lang),
        SourceId::Gleif => gleif::sample(lang),
        SourceId::Ror => ror::sample(lang),
        SourceId::Cldr => cldr::sample(lang),
        SourceId::NaturalEarth => natural_earth::sample(lang),
        SourceId::UdGermanGsd => ud_german_gsd::sample(),
        SourceId::Gnd => gnd::sample(lang),
        SourceId::Orcid => orcid::sample(lang),
        SourceId::Musicbrainz => musicbrainz::sample(lang),
        SourceId::Getty => getty::sample(lang),
        SourceId::Wiktionary => wiktionary::sample(lang),
        SourceId::Dbpedia => dbpedia::sample(lang),
        SourceId::Openstreetmap => openstreetmap::sample(region),
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
        return Err(format!("{source} does not support {kind:?}").into());
    }

    let records = match source {
        SourceId::Wikidata => wikidata::parse(bytes, lang)?,
        SourceId::Geonames => geonames::parse(bytes)?,
        SourceId::Discogs => discogs::parse(bytes)?,
        SourceId::Gleif => gleif::parse(bytes)?,
        SourceId::Ror => ror::parse(bytes)?,
        SourceId::Cldr => {
            // CLDR display names are locale-scoped, so the prepared plugin must
            // be tagged with a real locale rather than the "und" fallback.
            lang.ok_or("cldr prepare requires --lang")?;
            cldr::parse(bytes)?
        }
        SourceId::NaturalEarth => natural_earth::parse(bytes)?,
        SourceId::UdGermanGsd => ud_german_gsd::parse(bytes)?,
        SourceId::Gnd => gnd::parse(bytes)?,
        SourceId::Orcid => orcid::parse(bytes)?,
        SourceId::Musicbrainz => musicbrainz::parse(bytes)?,
        SourceId::Getty => getty::parse(bytes)?,
        SourceId::Wiktionary => wiktionary::parse(bytes, lang)?,
        SourceId::Dbpedia => dbpedia::parse(bytes)?,
        SourceId::Openstreetmap => openstreetmap::parse(bytes)?,
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
                .unwrap_or_else(|| format!("https://docs.invalid/textcase/sources/{source}")),
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
