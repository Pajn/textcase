use std::{collections::BTreeMap, path::Path};

use textcase::lexicon::{FstSidecar, PreparedLexicon, PreparedPayload, write_map, write_set};

pub fn build_fst_plugin(
    prepared: &PreparedLexicon,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let plugin = prepared.to_plugin_schema();
    let mut sidecar = FstSidecar {
        metadata: plugin.metadata,
        values: Vec::new(),
        candidate_values: Vec::new(),
    };

    match prepared.payload.clone() {
        PreparedPayload::WordSet(values) => {
            write_set(output, &values, &sidecar)?;
        }
        PreparedPayload::CanonicalMap(values)
        | PreparedPayload::MultiwordMap(values)
        | PreparedPayload::ProtectedForms(values) => {
            let indexed = index_string_values(values, &mut sidecar.values);
            write_map(output, &indexed, &sidecar)?;
        }
        PreparedPayload::RankedCandidates(values) => {
            let mut indexed = BTreeMap::new();
            for (key, candidates) in values {
                let index = sidecar.candidate_values.len() as u64;
                sidecar.candidate_values.push(candidates);
                indexed.insert(key, index);
            }
            write_map(output, &indexed, &sidecar)?;
        }
    }

    Ok(())
}

fn index_string_values(
    values: BTreeMap<String, String>,
    store: &mut Vec<String>,
) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for (key, value) in values {
        let index = store.len() as u64;
        store.push(value);
        out.insert(key, index);
    }
    out
}
