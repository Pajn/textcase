use std::collections::BTreeMap;

use crate::{prepare::normalize::normalized_aliases, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for record in records {
        if !record.canonical.contains(' ') {
            continue;
        }
        for alias in normalized_aliases(record) {
            if alias.contains(' ') {
                out.insert(alias, record.canonical.clone());
            }
        }
    }
    out
}
