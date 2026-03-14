#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use textcase::{
    CaseMode, CaseOptions, GermanMode, LexiconProvider, PluginSet,
    lexicon::{Candidate, FstSidecar, PreparedKind, PreparedLexicon, PreparedPayload, write_map},
    plugin::{LicenseMetadata, PluginKind, PluginMetadata, SchemaVersion, SourceMetadata},
};

pub fn sample_text() -> &'static str {
    "the rise of github - from berlin to new york and back again"
}

pub fn long_text() -> String {
    std::iter::repeat_n(sample_text(), 64)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn sentence_options<'a>() -> CaseOptions<'a> {
    CaseOptions {
        locale: "en",
        mode: CaseMode::Sentence,
        ..CaseOptions::default()
    }
}

pub fn title_options<'a>() -> CaseOptions<'a> {
    CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        ..CaseOptions::default()
    }
}

pub fn german_options<'a>(mode: GermanMode, lexicons: Option<&'a PluginSet>) -> CaseOptions<'a> {
    CaseOptions {
        locale: "de",
        mode: CaseMode::Sentence,
        german_mode: mode,
        lexicons: lexicons.map(|provider| provider as &dyn LexiconProvider),
        ..CaseOptions::default()
    }
}

pub fn canonical_map(count: usize) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for index in 0..count {
        let key = format!("entry-{index:05}");
        let value = format!("Entry {index:05}");
        map.insert(key.clone(), value);
        map.insert(format!("alias-{index:05}"), format!("Entry {index:05}"));
    }
    map
}

pub fn ranked_candidates_map(count: usize) -> BTreeMap<String, Vec<Candidate>> {
    let mut map = BTreeMap::new();
    for index in 0..count {
        map.insert(
            format!("wort-{index:04}"),
            vec![Candidate {
                value: format!("Wort {index:04}"),
                score: 10.0,
            }],
        );
    }
    map
}

pub fn json_plugin_bytes(count: usize) -> Vec<u8> {
    let prepared = PreparedLexicon {
        name: format!("benchmark-{count}"),
        kind: PreparedKind::CanonicalMap,
        locale: "en".to_string(),
        license: demo_license(),
        sources: vec![demo_source()],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload: PreparedPayload::CanonicalMap(canonical_map(count)),
    };
    serde_json::to_vec(&prepared.to_plugin_schema()).expect("plugin bytes")
}

pub fn ranked_json_plugin_bytes(count: usize) -> Vec<u8> {
    let prepared = PreparedLexicon {
        name: format!("benchmark-ranked-{count}"),
        kind: PreparedKind::RankedCandidates,
        locale: "de".to_string(),
        license: demo_license(),
        sources: vec![demo_source()],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload: PreparedPayload::RankedCandidates(ranked_candidates_map(count)),
    };
    serde_json::to_vec(&prepared.to_plugin_schema()).expect("plugin bytes")
}

pub fn fst_plugin_path(count: usize) -> PathBuf {
    static SMALL: OnceLock<PathBuf> = OnceLock::new();
    static MEDIUM: OnceLock<PathBuf> = OnceLock::new();
    static LARGE: OnceLock<PathBuf> = OnceLock::new();
    match count {
        128 => SMALL.get_or_init(|| build_fst_plugin(count)).clone(),
        1024 => MEDIUM.get_or_init(|| build_fst_plugin(count)).clone(),
        8192 => LARGE.get_or_init(|| build_fst_plugin(count)).clone(),
        other => build_fst_plugin(other),
    }
}

pub fn json_plugin_set(count: usize) -> PluginSet {
    PluginSet::from_json_bytes(&json_plugin_bytes(count)).expect("json plugin set")
}

pub fn fst_plugin_set(count: usize) -> PluginSet {
    PluginSet::from_fst_path(fst_plugin_path(count)).expect("fst plugin set")
}

pub fn german_ranked_plugin_set(count: usize) -> PluginSet {
    PluginSet::from_json_bytes(&ranked_json_plugin_bytes(count)).expect("ranked plugin set")
}

fn build_fst_plugin(count: usize) -> PathBuf {
    let path = unique_temp_path(&format!("textcase-bench-{count}"), "tclx");
    let values: Vec<String> = (0..count)
        .map(|index| format!("Entry {index:05}"))
        .collect();
    let mut index_map = BTreeMap::new();
    for index in 0..count {
        index_map.insert(format!("entry-{index:05}"), index as u64);
    }
    let sidecar = FstSidecar {
        metadata: PluginMetadata {
            schema: SchemaVersion::default(),
            name: format!("benchmark-fst-{count}"),
            kind: PluginKind::CanonicalMap,
            locales: vec!["en".to_string()],
            license: demo_license(),
            sources: vec![demo_source()],
            generated_at: "1970-01-01T00:00:00Z".to_string(),
            checksum: "benchmark".to_string(),
        },
        values,
        candidate_values: Vec::new(),
    };
    write_map(&path, &index_map, &sidecar).expect("write fst plugin");
    path
}

fn demo_license() -> LicenseMetadata {
    LicenseMetadata {
        name: "CC0".to_string(),
        summary: "benchmark fixture".to_string(),
        acknowledgement_flag: None,
    }
}

fn demo_source() -> SourceMetadata {
    SourceMetadata {
        id: "benchmark".to_string(),
        display_name: "Benchmark".to_string(),
        url: "https://example.invalid/benchmark".to_string(),
        version: "1".to_string(),
        class: "green".to_string(),
    }
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = env::temp_dir().join("textcase-benches");
    fs::create_dir_all(&dir).expect("bench temp dir");
    dir.join(format!("{prefix}-{nanos}.{extension}"))
}
