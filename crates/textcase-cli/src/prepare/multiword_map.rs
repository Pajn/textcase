use std::collections::BTreeMap;

use crate::{prepare::normalize::lookup_entries, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for record in records {
        for (key, value) in lookup_entries(record) {
            if key.contains(' ') && value.contains(' ') {
                out.insert(key, value);
            }
        }
    }
    out
}
