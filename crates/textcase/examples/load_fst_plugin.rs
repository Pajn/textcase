use std::collections::BTreeMap;
use std::env;
use std::fs;

use textcase::lexicon::{FstSidecar, write_map};
use textcase::plugin::{
    LicenseMetadata, PluginKind, PluginMetadata, SchemaVersion, SourceMetadata,
};
use textcase::{CaseOptions, PluginSet, convert};

fn main() {
    let path = env::temp_dir().join("demo-textcase.tclx");
    let sidecar = FstSidecar {
        metadata: PluginMetadata {
            schema: SchemaVersion::default(),
            name: "demo-fst".to_string(),
            kind: PluginKind::CanonicalMap,
            locales: vec!["en".to_string()],
            license: LicenseMetadata {
                name: "CC0".to_string(),
                summary: "demo".to_string(),
                acknowledgement_flag: None,
            },
            sources: vec![SourceMetadata {
                id: "demo".to_string(),
                display_name: "Demo".to_string(),
                url: "https://example.invalid".to_string(),
                version: "1".to_string(),
                class: "green".to_string(),
            }],
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
    println!("{}", convert("github in berlin", &options));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(path.with_file_name("demo-textcase.tclx.meta.json"));
}
