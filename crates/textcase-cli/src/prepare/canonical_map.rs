use std::collections::BTreeMap;

use crate::{prepare::normalize::lookup_entries, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for record in records {
        for (key, value) in lookup_entries(record) {
            out.insert(key, value);
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
