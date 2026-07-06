use std::collections::BTreeMap;

use textcase::lexicon::Candidate;

use crate::{prepare::normalize::lookup_entries, sources::SourceRecord};

pub fn build(records: &[SourceRecord]) -> BTreeMap<String, Vec<Candidate>> {
    let mut out: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    for record in records {
        // An inflected form keeps its own casing as the candidate value
        // ("probleme" recases to "Probleme"), rather than being replaced by
        // the lemma ("Problem"), via the lookup-entry rewrite guard.
        for (key, value) in lookup_entries(record) {
            out.entry(key).or_default().push(Candidate {
                value,
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
