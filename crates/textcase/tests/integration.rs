use std::{
    collections::BTreeMap,
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

use textcase::lexicon::{FstSidecar, PreparedKind, PreparedLexicon, PreparedPayload, write_map};
use textcase::plugin::{
    LicenseMetadata, PluginKind, PluginMetadata, SchemaVersion, SourceMetadata,
};
use textcase::{
    CaseMode, CaseOptions, GermanMode, PluginSet, SubtitleSeparatorStyle, convert, sentence_case,
};

#[test]
fn sentence_case_normalizes_basic_english() {
    assert_eq!(
        sentence_case("the rise of github in berlin", "en"),
        "The rise of GitHub in berlin"
    );
}

#[test]
fn sentence_case_downcases_title_cased_input() {
    // Ordinary capitalized words must not be treated as mixed case, otherwise
    // converting already-title-cased text would be a no-op.
    assert_eq!(
        sentence_case("The Quick Brown Fox Jumps", "en"),
        "The quick brown fox jumps"
    );
}

#[test]
fn sentence_case_preserves_internal_capitals() {
    assert_eq!(
        sentence_case("the iPhone and the McDonald empire", "en"),
        "The iPhone and the McDonald empire"
    );
}

#[test]
fn sentence_title_capitalizes_after_subtitle_separator() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("the rise of github - inside rust tooling", &options),
        "The rise of GitHub: Inside Rust tooling"
    );
}

#[test]
fn title_case_keeps_small_words_lowercase() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("war and peace in europe", &options),
        "War and Peace in Europe"
    );
}

#[test]
fn json_plugin_restores_known_forms() {
    let prepared = PreparedLexicon {
        name: "demo".to_string(),
        kind: PreparedKind::CanonicalMap,
        locale: "en".to_string(),
        license: LicenseMetadata {
            name: "CC0".to_string(),
            summary: "demo".to_string(),
            acknowledgement_flag: None,
        },
        sources: vec![SourceMetadata {
            id: "demo".to_string(),
            display_name: "Demo".to_string(),
            url: "https://example.invalid".to_string(),
            version: "1".to_string(),
            class: "green".to_string(),
        }],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload: PreparedPayload::CanonicalMap(BTreeMap::from([
            ("berlin".to_string(), "Berlin".to_string()),
            ("new york".to_string(), "New York".to_string()),
        ])),
    };
    let bytes = serde_json::to_vec(&prepared.to_plugin_schema()).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions {
        locale: "en",
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("berlin and new york", &options),
        "Berlin and New York"
    );
}

#[test]
fn fst_plugin_round_trip_restores_forms() {
    let path = unique_temp_path("textcase-fst", "tclx");
    let sidecar = FstSidecar {
        metadata: PluginMetadata {
            schema: SchemaVersion::default(),
            name: "demo".to_string(),
            kind: PluginKind::CanonicalMap,
            locales: vec!["en".to_string()],
            license: LicenseMetadata {
                name: "CC0".to_string(),
                summary: "demo".to_string(),
                acknowledgement_flag: None,
            },
            sources: vec![SourceMetadata {
                id: "demo".to_string(),
                display_name: "Demo".to_string(),
                url: "https://example.invalid".to_string(),
                version: "1".to_string(),
                class: "green".to_string(),
            }],
            generated_at: "1970-01-01T00:00:00Z".to_string(),
            checksum: "demo".to_string(),
        },
        values: vec!["Berlin".to_string(), "GitHub".to_string()],
        candidate_values: Vec::new(),
    };
    let mut map = BTreeMap::new();
    map.insert("berlin".to_string(), 0);
    map.insert("github".to_string(), 1);
    write_map(&path, &map, &sidecar).unwrap();

    let lexicons = PluginSet::from_fst_path(&path).unwrap();
    let options = CaseOptions {
        locale: "en",
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    assert_eq!(convert("github in berlin", &options), "GitHub in Berlin");

    fs::remove_file(&path).unwrap();
    fs::remove_file(path.with_file_name(format!(
        "{}.meta.json",
        path.file_name().unwrap().to_string_lossy()
    )))
    .unwrap();
}

#[test]
fn german_balanced_mode_recovers_common_noun_context() {
    let conservative = CaseOptions {
        locale: "de",
        german_mode: GermanMode::Conservative,
        ..CaseOptions::default()
    };
    let balanced = CaseOptions {
        locale: "de",
        german_mode: GermanMode::Balanced,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("ich mag die wissenschaft", &conservative),
        "Ich mag die wissenschaft"
    );
    assert_eq!(
        convert("ich mag die wissenschaft", &balanced),
        "Ich mag die Wissenschaft"
    );
}

#[test]
fn german_aggressive_mode_uses_ranked_candidates() {
    let prepared = PreparedLexicon {
        name: "de-ranked".to_string(),
        kind: PreparedKind::RankedCandidates,
        locale: "de".to_string(),
        license: LicenseMetadata {
            name: "CC0".to_string(),
            summary: "demo".to_string(),
            acknowledgement_flag: None,
        },
        sources: vec![SourceMetadata {
            id: "demo".to_string(),
            display_name: "Demo".to_string(),
            url: "https://example.invalid".to_string(),
            version: "1".to_string(),
            class: "green".to_string(),
        }],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload: PreparedPayload::RankedCandidates(BTreeMap::from([
            (
                "sprache".to_string(),
                vec![textcase::Candidate {
                    value: "Sprache".to_string(),
                    score: 5.0,
                }],
            ),
            (
                "analyse".to_string(),
                vec![textcase::Candidate {
                    value: "Analyse".to_string(),
                    score: 4.0,
                }],
            ),
        ])),
    };
    let bytes = serde_json::to_vec(&prepared.to_plugin_schema()).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions {
        locale: "de",
        german_mode: GermanMode::Aggressive,
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("sprache und analyse", &options),
        "Sprache und Analyse"
    );
}

fn unique_temp_path(prefix: &str, extension: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{nanos}.{extension}"))
}
