use textcase::{convert, CaseOptions, PluginSet};

fn main() {
    let plugin = serde_json::json!({
        "schema": { "major": 1, "minor": 0 },
        "name": "demo-canonical-map",
        "kind": "canonical-map",
        "locales": ["en"],
        "license": { "name": "CC0", "summary": "demo", "acknowledgement_flag": null },
        "sources": [{ "id": "demo", "display_name": "Demo", "url": "https://example.invalid", "version": "1", "class": "green" }],
        "generated_at": "1970-01-01T00:00:00Z",
        "checksum": "demo",
        "payload_kind": "canonical-map",
        "payload": { "berlin": "Berlin", "github": "GitHub" }
    });
    let bytes = serde_json::to_vec(&plugin).unwrap();
    let lexicons = PluginSet::from_json_bytes(&bytes).unwrap();
    let options = CaseOptions { locale: "en", lexicons: Some(&lexicons), ..CaseOptions::default() };
    println!("{}", convert("github in berlin", &options));
}
