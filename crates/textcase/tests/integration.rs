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
fn unknown_locale_gets_neutral_profile_not_english() {
    // English stop words must not leak into languages without a profile.
    let polish = CaseOptions {
        locale: "pl",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(convert("all of the things", &polish), "All Of The Things");
    let english = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(convert("all of the things", &english), "All of the Things");
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
fn sentence_case_preserves_mid_sentence_proper_nouns() {
    // A lone capital in an otherwise lowercase sentence is a proper-noun
    // signal that no lexicon can restore once lost.
    assert_eq!(
        sentence_case("yesterday Alice met Bob in Paris", "en"),
        "Yesterday Alice met Bob in Paris"
    );
    // German prose keeps its capitalized nouns even in conservative mode.
    assert_eq!(
        sentence_case("wir besuchen die Alte Oper heute", "de"),
        "Wir besuchen die Alte Oper heute"
    );
}

#[test]
fn sentence_case_can_disable_existing_capital_preservation() {
    let options = CaseOptions {
        locale: "en",
        preserve_existing_capitals: false,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("yesterday Alice met Bob", &options),
        "Yesterday alice met bob"
    );
}

#[test]
fn sentence_case_capitalizes_english_pronoun_i() {
    assert_eq!(sentence_case("i think i can", "en"), "I think I can");
    assert_eq!(
        sentence_case("he said i'm ready", "en"),
        "He said I'm ready"
    );
    assert_eq!(sentence_case("I THINK I CAN", "en"), "I think I can");
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
fn french_title_keeps_elided_particles_lowercase() {
    let options = CaseOptions {
        locale: "fr",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    // Mid-title the elided particle stays lowercase; the title-opening word
    // still starts with a capital.
    assert_eq!(
        convert("l'homme d'affaires et la vie", &options),
        "L'Homme d'Affaires et la Vie"
    );
    // French sentence case is unaffected: ICU capitalizes the whole first
    // word segment.
    assert_eq!(
        sentence_case("l'homme est arrivé", "fr"),
        "L'homme est arrivé"
    );
}

#[test]
fn sentence_case_keeps_leading_contraction() {
    assert_eq!(sentence_case("i'm going now", "en"), "I'm going now");
}

#[test]
fn sentence_case_does_not_split_hyphenated_word() {
    // Sentence case capitalizes only the first letter of the sentence, so an
    // intra-word hyphen must not trigger a second capital.
    assert_eq!(
        sentence_case("re-enter the code", "en"),
        "Re-enter the code"
    );
    assert_eq!(sentence_case("jean-paul left", "en"), "Jean-paul left");
}

#[test]
fn title_case_splits_hyphenated_names() {
    // Title mode still capitalizes each hyphen-separated part.
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(convert("jean-paul sartre", &options), "Jean-Paul Sartre");
    assert_eq!(convert("coca-cola", &options), "Coca-Cola");
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
fn sentence_case_numeric_abbreviations_require_a_number() {
    // "no." is an abbreviation only directly before a number...
    assert_eq!(
        sentence_case("she wrote no. 5 on the door", "en"),
        "She wrote no. 5 on the door"
    );
    // ...elsewhere it is ordinary prose and its period ends the sentence.
    assert_eq!(
        sentence_case("the answer is no. it was clear", "en"),
        "The answer is no. It was clear"
    );
}

#[test]
fn sentence_case_trailing_abbreviations_follow_next_word_casing() {
    // Mid-phrase, "inc." continues the sentence...
    assert_eq!(
        sentence_case("we sued acme inc. yesterday it settled", "en"),
        "We sued acme inc. yesterday it settled"
    );
    // ...but a capitalized next word marks a real sentence end after "etc.".
    assert_eq!(
        sentence_case("apples, pears, etc. Then we left", "en"),
        "Apples, pears, etc. Then we left"
    );
}

#[test]
fn abbreviations_are_locale_specific() {
    // German "Nr." abbreviates before a number...
    assert_eq!(
        sentence_case("siehe nr. 5 bitte", "de"),
        "Siehe nr. 5 bitte"
    );
    // ...and "usw." continues the phrase before a lowercase word.
    assert_eq!(
        sentence_case("äpfel, birnen usw. dann gingen wir", "de"),
        "Äpfel, birnen usw. dann gingen wir"
    );
    // English has no "usw."; the period is a real terminal there.
    assert_eq!(
        sentence_case("äpfel, birnen usw. dann gingen wir", "en"),
        "Äpfel, birnen usw. Dann gingen wir"
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
fn sentence_case_splits_on_unspaced_exclamation_and_question() {
    // Unlike the period, "!" and "?" are never decimal points or abbreviation
    // dots, so they end the sentence even without a following space.
    assert_eq!(
        sentence_case("wait!something happened", "en"),
        "Wait!Something happened"
    );
    assert_eq!(sentence_case("really?yes it is", "en"), "Really?Yes it is");
}

#[test]
fn sentence_case_ellipsis_follows_input_casing() {
    // A trailing-off ellipsis continues the sentence...
    assert_eq!(
        sentence_case("wait… there is more", "en"),
        "Wait… there is more"
    );
    assert_eq!(
        sentence_case("hold on... something is coming", "en"),
        "Hold on... something is coming"
    );
    // ...but a capitalized next word marks a genuine new sentence.
    assert_eq!(
        sentence_case("wait… Then it began", "en"),
        "Wait… Then it began"
    );
    assert_eq!(
        sentence_case("hold on... Something is coming", "en"),
        "Hold on... Something is coming"
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
fn sentence_case_shouting_tolerates_lowercase_stop_words() {
    // A lowercase connective does not turn a shouted title into acronyms.
    assert_eq!(
        sentence_case("NEW YORK vs THE WORLD", "en"),
        "New York vs the world"
    );
    // Short all-caps words without a long shouted word stay acronyms.
    assert_eq!(sentence_case("USA vs USSR", "en"), "USA vs USSR");
}

#[test]
fn sentence_case_shouting_is_scoped_per_sentence() {
    // The first sentence is shouted and converts; the second keeps its
    // genuine acronym.
    assert_eq!(
        sentence_case("BREAKING NEWS TODAY. the NASA probe landed", "en"),
        "Breaking news today. The NASA probe landed"
    );
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
fn sentence_case_gates_ambiguous_builtin_forms_on_casing() {
    // "rust" is also an ordinary word; without a casing signal in the input
    // the builtin canonical form must not fire.
    assert_eq!(
        sentence_case("the rust on the old pipe", "en"),
        "The rust on the old pipe"
    );
    // A cased occurrence restores the canonical form.
    assert_eq!(
        sentence_case("we love Rust dearly", "en"),
        "We love Rust dearly"
    );
    // A title always carries the signal.
    let title = CaseOptions {
        locale: "en",
        mode: CaseMode::Title,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("programming in rust", &title),
        "Programming in Rust"
    );

    // "latex" is likewise an ordinary word (the material); it stays lowercase in
    // plain prose but restores under a casing signal.
    assert_eq!(
        sentence_case("the latex gloves tore", "en"),
        "The latex gloves tore"
    );
    assert_eq!(
        convert("typesetting in latex", &title),
        "Typesetting in LaTeX"
    );
}

#[test]
fn user_lexicon_overrides_builtin_forms() {
    let prepared = demo_prepared_lexicon(
        "en-override",
        PreparedKind::CanonicalMap,
        "en",
        PreparedPayload::CanonicalMap(BTreeMap::from([(
            "github".to_string(),
            "GITHUB".to_string(),
        )])),
    );
    let bytes = serde_json::to_vec(&prepared.to_plugin_schema()).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions {
        locale: "en",
        lexicons: Some(&lexicons),
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("using github daily", &options),
        "Using GITHUB daily"
    );
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
fn sentence_title_ignores_ranges_and_attached_colons() {
    // "a - f" is a range, not a subtitle break.
    assert_eq!(
        sentence_case_title("grades a - f explained", "en"),
        "Grades a - f explained"
    );
    // An unspaced colon is a time, ratio, or brand, not a subtitle separator.
    assert_eq!(
        sentence_case_title("re:invent recap", "en"),
        "Re:invent recap"
    );
}

#[test]
fn sentence_title_capitalizes_after_attached_em_dash() {
    assert_eq!(
        sentence_case_title("the album—deluxe edition", "en"),
        "The album—Deluxe edition"
    );
}

#[test]
fn subtitle_normalization_ignores_letter_ranges() {
    let options = CaseOptions {
        locale: "en",
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    assert_eq!(
        convert("grades a - f explained", &options),
        "Grades a - f explained"
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
    // Lowercase "rust" carries no casing signal in a sentence-like mode, so
    // the ambiguous builtin form is not restored.
    assert_eq!(
        convert("the rise of github - inside rust tooling", &options),
        "The rise of GitHub: Inside rust tooling"
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
fn german_aggressive_mode_ignores_noise_candidates() {
    let prepared = demo_prepared_lexicon(
        "de-ranked",
        PreparedKind::RankedCandidates,
        "de",
        PreparedPayload::RankedCandidates(BTreeMap::from([(
            "haus".to_string(),
            vec![textcase::Candidate {
                value: "HAUS".to_string(),
                score: 0.3,
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
    // The below-threshold candidate is ignored, and the balanced article
    // heuristic still capitalizes the noun.
    assert_eq!(
        convert("wir sehen das haus", &options),
        "Wir sehen das Haus"
    );
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
