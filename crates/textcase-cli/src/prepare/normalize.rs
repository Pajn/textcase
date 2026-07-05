use crate::sources::SourceRecord;

/// Lookup entries for a record, guarding against text rewrites.
///
/// The canonical form and its spelling variants (casing, diacritics,
/// German-style transliteration, punctuation and spacing) map to the
/// canonical. An alias that is a *different name* — a translation, a previous
/// name, an inflected form — maps to its own casing instead, and is dropped
/// when it carries none: mapping "berlim" to "Berlin" would rewrite the text,
/// not recase it.
pub fn lookup_entries(record: &SourceRecord) -> Vec<(String, String)> {
    let mut entries = vec![(record.canonical.to_lowercase(), record.canonical.clone())];
    // Fold the canonical once and reuse it for every alias check.
    let canonical_forms = folded_forms(&record.canonical);
    for alias in &record.aliases {
        let key = alias.to_lowercase();
        if matches_folded(alias, &canonical_forms) {
            entries.push((key, record.canonical.clone()));
        } else if *alias != key {
            entries.push((key, alias.clone()));
        }
    }
    entries.sort();
    entries.dedup();
    entries
}

/// Whether `alias` is a spelling of the same word sequence as the already-folded
/// canonical `(stripped, transliterated)` forms — differing only in casing,
/// punctuation, spacing, diacritics, or Germanic transliteration ("Muenchen" for
/// "München") — on any of the four cross-comparisons.
fn matches_folded(alias: &str, canonical: &(String, String)) -> bool {
    let (alias_stripped, alias_transliterated) = folded_forms(alias);
    alias_stripped == canonical.0
        || alias_stripped == canonical.1
        || alias_transliterated == canonical.0
        || alias_transliterated == canonical.1
}

/// Folds a name two ways: diacritics stripped to base letters ("ü" → "u") and
/// diacritics expanded Germanic-style ("ü" → "ue"). Both forms are lowercase
/// with punctuation and whitespace removed.
fn folded_forms(input: &str) -> (String, String) {
    let mut stripped = String::with_capacity(input.len());
    let mut transliterated = String::with_capacity(input.len());
    for ch in input.chars().flat_map(char::to_lowercase) {
        if !ch.is_alphanumeric() {
            continue;
        }
        match strip_diacritic(ch) {
            Some(base) => stripped.push_str(base),
            None => stripped.push(ch),
        }
        match expand_diacritic(ch) {
            Some(expanded) => transliterated.push_str(expanded),
            None => transliterated.push(ch),
        }
    }
    (stripped, transliterated)
}

fn strip_diacritic(ch: char) -> Option<&'static str> {
    Some(match ch {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'ç' | 'ć' | 'č' => "c",
        'ď' | 'đ' | 'ð' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => "e",
        'ğ' | 'ģ' => "g",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i",
        'ķ' => "k",
        'ł' | 'ļ' | 'ľ' => "l",
        'ñ' | 'ń' | 'ņ' | 'ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => "o",
        'ř' => "r",
        'ś' | 'ş' | 'š' | 'ș' => "s",
        'ť' | 'ț' => "t",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => "u",
        'ý' | 'ÿ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'þ' => "th",
        _ => return None,
    })
}

fn expand_diacritic(ch: char) -> Option<&'static str> {
    Some(match ch {
        'ä' => "ae",
        'ö' | 'ø' => "oe",
        'ü' => "ue",
        'å' => "aa",
        _ => return strip_diacritic(ch),
    })
}

#[cfg(test)]
mod tests {
    use super::{folded_forms, lookup_entries, matches_folded};
    use crate::sources::SourceRecord;

    fn is_spelling_variant(alias: &str, canonical: &str) -> bool {
        matches_folded(alias, &folded_forms(canonical))
    }

    #[test]
    fn detects_spelling_variants() {
        assert!(is_spelling_variant("bjork", "Björk"));
        assert!(is_spelling_variant("Muenchen", "München"));
        assert!(is_spelling_variant("Sao Paulo", "São Paulo"));
        assert!(is_spelling_variant("NWA", "N.W.A"));
        assert!(is_spelling_variant("cote d'ivoire", "Côte d'Ivoire"));
        assert!(!is_spelling_variant("Berlim", "Berlin"));
        assert!(!is_spelling_variant("Munich", "München"));
        assert!(!is_spelling_variant("Probleme", "Problem"));
    }

    #[test]
    fn variant_aliases_restore_the_canonical() {
        let record = SourceRecord {
            canonical: "Björk".to_string(),
            aliases: vec!["bjork".to_string()],
            score: 1.0,
        };
        assert_eq!(
            lookup_entries(&record),
            vec![
                ("bjork".to_string(), "Björk".to_string()),
                ("björk".to_string(), "Björk".to_string()),
            ]
        );
    }

    #[test]
    fn different_names_restore_their_own_casing() {
        let record = SourceRecord {
            canonical: "Volkswagen Aktiengesellschaft".to_string(),
            aliases: vec!["Volkswagen AG".to_string()],
            score: 1.0,
        };
        let entries = lookup_entries(&record);
        assert!(entries.contains(&("volkswagen ag".to_string(), "Volkswagen AG".to_string())));
        assert!(!entries.contains(&(
            "volkswagen ag".to_string(),
            "Volkswagen Aktiengesellschaft".to_string()
        )));
    }

    #[test]
    fn caseless_different_names_are_dropped() {
        let record = SourceRecord {
            canonical: "Berlin".to_string(),
            aliases: vec!["berlim".to_string()],
            score: 1.0,
        };
        assert_eq!(
            lookup_entries(&record),
            vec![("berlin".to_string(), "Berlin".to_string())]
        );
    }
}
