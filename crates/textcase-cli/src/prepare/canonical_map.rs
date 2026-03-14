use std::collections::BTreeMap;

use crate::{prepare::normalize::normalized_aliases, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for record in records {
        for alias in normalized_aliases(record) {
            out.insert(alias, record.canonical.clone());
        }
    }
    out
}

pub fn build_protected(records: &[SourceRecord]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for record in records {
        if record.canonical.chars().any(|ch| ch.is_lowercase())
            && record.canonical.chars().any(|ch| ch.is_uppercase())
        {
            out.insert(record.canonical.to_lowercase(), record.canonical.clone());
        }
    }
    out
}
