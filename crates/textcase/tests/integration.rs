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
    sentence_case_title,
};

#[test]
fn sentence_case_splits_on_cjk_terminals() {
    // A CJK full stop starts a new sentence even without a following space, so
    // the next Latin word is capitalized.
    assert_eq!(
        sentence_case("你好。hello world", "ja"),
        "你好。Hello world"
    );
    assert_eq!(
        sentence_case("really？yes indeed", "en"),
        "Really？Yes indeed"
    );
}

#[test]
fn sentence_case_splits_on_devanagari_danda() {
    assert_eq!(
        sentence_case("namaste। hello there", "hi"),
        "Namaste। Hello there"
    );
}

#[test]
fn dutch_titlecases_ij_digraph() {
    // ICU applies the Dutch "IJ" rule for the nl locale...
    assert_eq!(
        sentence_case("ijsselmeer is a lake", "nl"),
        "IJsselmeer is a lake"
    );
    // ...but not for other locales.
    assert_eq!(
        sentence_case("ijsselmeer is a lake", "en"),
        "Ijsselmeer is a lake"
    );
}

#[test]
fn turkish_uses_dotted_capital_i() {
    assert_eq!(sentence_case("istanbul", "tr"), "İstanbul");
    assert_eq!(sentence_case("ışık parlak", "tr"), "Işık parlak");
}

#[test]
fn greek_lowercases_to_final_sigma() {
    assert_eq!(sentence_case("ΤΟ ΟΔΟΣ", "el"), "Το οδος");
}

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
fn sentence_case_keeps_contractions_intact() {
    assert_eq!(
        sentence_case("don't stop me now", "en"),
        "Don't stop me now"
    );
}

#[test]
fn sentence_case_recases_quoted_known_forms() {
    // The trailing quote must not be glued onto the word, or the lexicon
    // lookup for "github" fails.
    assert_eq!(
        sentence_case("she said 'github' loudly", "en"),
        "She said 'GitHub' loudly"
    );
}

#[test]
fn title_case_capitalizes_single_letter_apostrophe_prefix() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(convert("the o'brien files", &options), "The O'Brien Files");
}

#[test]
fn sentence_case_keeps_leading_contraction() {
    assert_eq!(sentence_case("i'm going now", "en"), "I'm going now");
}

#[test]
fn title_case_keeps_single_letter_contractions() {
    // A single-letter prefix followed by a contraction tail stays one word,
    // unlike the O'Brien name-particle split above.
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("i'm i'll i've y'all", &options),
        "I'm I'll I've Y'all"
    );
}

#[test]
fn sentence_case_does_not_split_on_abbreviations() {
    assert_eq!(
        sentence_case("visit dr. smith e.g. tomorrow please", "en"),
        "Visit dr. smith e.g. tomorrow please"
    );
}

#[test]
fn sentence_case_splits_on_real_terminals() {
    assert_eq!(
        sentence_case("the show ended. everyone left", "en"),
        "The show ended. Everyone left"
    );
}

#[test]
fn sentence_case_ignores_decimal_points() {
    assert_eq!(
        sentence_case("version 3.5 is ready", "en"),
        "Version 3.5 is ready"
    );
}

#[test]
fn sentence_case_converts_shouting_titles() {
    assert_eq!(sentence_case("MY GREAT ALBUM", "en"), "My great album");
}

#[test]
fn sentence_case_preserves_acronyms_in_mixed_text() {
    assert_eq!(
        sentence_case("NASA launched the probe", "en"),
        "NASA launched the probe"
    );
}

#[test]
fn sentence_case_lexicon_overrides_all_caps() {
    // A known canonical form wins over acronym preservation...
    assert_eq!(sentence_case("the GITHUB repo", "en"), "The GitHub repo");
    // ...while an all-caps word absent from the lexicon stays preserved.
    assert_eq!(sentence_case("the NASA probe", "en"), "The NASA probe");
}

#[test]
fn sentence_case_all_caps_phrase_canonicalizes() {
    // Same precedence for multi-word canonical forms.
    assert_eq!(sentence_case("NEW YORK is big", "en"), "New York is big");
}

#[test]
fn sentence_case_recases_stray_single_capital() {
    assert_eq!(sentence_case("buy A dog", "en"), "Buy a dog");
}

#[test]
fn sentence_case_preserves_line_breaks() {
    assert_eq!(
        sentence_case("line one\nline two", "en"),
        "Line one\nline two"
    );
}

#[test]
fn sentence_case_still_collapses_horizontal_whitespace() {
    assert_eq!(sentence_case("hello    world", "en"), "Hello world");
}

#[test]
fn sentence_title_flattens_line_breaks() {
    assert_eq!(
        sentence_case_title("line one\nline two", "en"),
        "Line one line two"
    );
}

#[test]
fn sentence_case_does_not_capitalize_after_colon() {
    // A colon in running prose does not start a new sentence.
    assert_eq!(
        sentence_case("note: this is important", "en"),
        "Note: this is important"
    );
}

#[test]
fn sentence_title_still_capitalizes_after_colon() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("note: this is important", &options),
        "Note: This is important"
    );
}

#[test]
fn title_case_capitalizes_first_word_after_colon() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("something: the reckoning", &options),
        "Something: The Reckoning"
    );
}

#[test]
fn subtitle_normalization_ignores_numeric_ranges() {
    let options = CaseOptions {
        locale: "en",
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("tracks 3 - 5 remastered", &options),
        "Tracks 3 - 5 remastered"
    );
}

#[test]
fn subtitle_normalization_preserves_literal_sentinel_text() {
    let options = CaseOptions {
        locale: "en",
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(convert("a <subtitle> b", &options), "A <subtitle> b");
}

#[test]
fn subtitle_normalization_converts_flanked_dash() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("the album - remastered", &options),
        "The album: Remastered"
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
fn sentence_case_capitalizes_phrase_at_sentence_start() {
    // The canonical form starts with a lowercase particle; it must still be
    // capitalized when it begins the sentence.
    assert_eq!(
        sentence_case("van der waals forces are weak", "en"),
        "Van der Waals forces are weak"
    );
}

#[test]
fn sentence_case_keeps_phrase_particle_lowercase_mid_sentence() {
    assert_eq!(
        sentence_case("we study van der waals forces", "en"),
        "We study van der Waals forces"
    );
}

#[test]
fn german_aggressive_mode_still_capitalizes_sentence_start() {
    // A lexicon whose canonical form for a sentence-initial word is lowercase
    // must not defeat sentence-start capitalization.
    let prepared = demo_prepared_lexicon(
        "de-ranked",
        PreparedKind::RankedCandidates,
        "de",
        PreparedPayload::RankedCandidates(BTreeMap::from([(
            "wir".to_string(),
            vec![textcase::Candidate {
                value: "wir".to_string(),
                score: 5.0,
            }],
        )])),
    );
    let bytes = serde_json::to_vec(&prepared.to_plugin_schema()).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions {
        locale: "de",
        german_mode: GermanMode::Aggressive,
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    assert_eq!(convert("wir sind hier", &options), "Wir sind hier");
}

#[test]
fn german_title_capitalizes_after_colon_over_lexicon() {
    let prepared = demo_prepared_lexicon(
        "de-ranked",
        PreparedKind::RankedCandidates,
        "de",
        PreparedPayload::RankedCandidates(BTreeMap::from([(
            "wort".to_string(),
            vec![textcase::Candidate {
                value: "wort".to_string(),
                score: 5.0,
            }],
        )])),
    );
    let bytes = serde_json::to_vec(&prepared.to_plugin_schema()).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions {
        locale: "de",
        mode: CaseMode::Title,
        german_mode: GermanMode::Aggressive,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    // "wort" opens the subtitle, so it must be capitalized even though the
    // lexicon's canonical form for it is lowercase.
    assert_eq!(
        convert("das buch: wort zaehlt", &options),
        "Das Buch: Wort Zaehlt"
    );
}

#[test]
fn json_plugin_restores_known_forms() {
    let prepared = demo_prepared_lexicon(
        "demo",
        PreparedKind::CanonicalMap,
        "en",
        PreparedPayload::CanonicalMap(BTreeMap::from([
            ("berlin".to_string(), "Berlin".to_string()),
            ("new york".to_string(), "New York".to_string()),
        ])),
    );
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
            license: demo_license(),
            sources: vec![demo_source()],
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
fn german_balanced_mode_does_not_carry_article_across_punctuation() {
    let balanced = CaseOptions {
        locale: "de",
        german_mode: GermanMode::Balanced,
        ..CaseOptions::default()
    };
    // "der" is an article, but the comma ends the phrase, so the adverb
    // "gestern" must not be capitalized as if it were the article's noun.
    assert_eq!(
        convert("der, gestern kamen wir", &balanced),
        "Der, gestern kamen wir"
    );
}

#[test]
fn german_aggressive_mode_uses_ranked_candidates() {
    let prepared = demo_prepared_lexicon(
        "de-ranked",
        PreparedKind::RankedCandidates,
        "de",
        PreparedPayload::RankedCandidates(BTreeMap::from([
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
    );
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

fn demo_license() -> LicenseMetadata {
    LicenseMetadata {
        name: "CC0".to_string(),
        summary: "demo".to_string(),
        acknowledgement_flag: None,
    }
}

fn demo_source() -> SourceMetadata {
    SourceMetadata {
        id: "demo".to_string(),
        display_name: "Demo".to_string(),
        url: "https://example.invalid".to_string(),
        version: "1".to_string(),
        class: "green".to_string(),
    }
}

fn demo_prepared_lexicon(
    name: &str,
    kind: PreparedKind,
    locale: &str,
    payload: PreparedPayload,
) -> PreparedLexicon {
    PreparedLexicon {
        name: name.to_string(),
        kind,
        locale: locale.to_string(),
        license: demo_license(),
        sources: vec![demo_source()],
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        payload,
    }
}

fn unique_temp_path(prefix: &str, extension: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{nanos}.{extension}"))
}
