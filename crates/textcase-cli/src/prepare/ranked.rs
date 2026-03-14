use std::collections::BTreeMap;

use textcase::lexicon::Candidate;

use crate::{prepare::normalize::normalized_aliases, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, Vec<Candidate>> {
    let mut out: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    for record in records {
        for alias in normalized_aliases(record) {
            out.entry(alias).or_default().push(Candidate {
                value: record.canonical.clone(),
                score: record.score,
            });
        }
    }

    for candidates in out.values_mut() {
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.value.cmp(&right.value))
        });
        candidates.dedup_by(|left, right| left.value == right.value);
    }

    out
}
