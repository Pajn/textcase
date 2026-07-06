use std::{
    env, fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn list_sources_includes_classes() {
    let output = Command::new(bin())
        .args(["lexicon", "list-sources"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wikidata"));
    assert!(stdout.contains("green"));
    assert!(stdout.contains("ud-german-gsd"));
    for source in ["discogs", "gleif", "ror", "cldr", "natural-earth"] {
        assert!(stdout.contains(source), "missing source: {source}");
    }
    assert!(!stdout.contains("omw"));
}

#[test]
fn show_license_reports_acknowledgement_requirements() {
    let output = Command::new(bin())
        .args(["lexicon", "show-license", "ud-german-gsd"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--acknowledge-cc-by-sa"));
    assert!(stdout.contains("fetch guidance:"));
}

#[test]
fn orange_source_requires_acknowledgement() {
    let dir = unique_temp_dir("textcase-cli-orange");
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(bin())
        .args(["lexicon", "fetch", "ud-german-gsd", "--output-dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--acknowledge-cc-by-sa"));
}

#[test]
fn unsupported_source_requires_documented_url() {
    let dir = unique_temp_dir("textcase-cli-fetch-gap");
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(bin())
        .args([
            "lexicon",
            "fetch",
            "wikidata",
            "--lang",
            "en",
            "--output-dir",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not have a built-in fetch workflow"));
    assert!(stderr.contains("--url"));
}

#[test]
fn prepare_build_and_inspect_json_plugin() {
    let dir = unique_temp_dir("textcase-cli-json");
    let raw = dir.join("raw");
    let prepared = dir.join("prepared.json");
    let plugin = dir.join("plugin.json");
    fs::create_dir_all(&raw).unwrap();

    assert!(
        Command::new(bin())
            .args([
                "lexicon",
                "fetch",
                "wikidata",
                "--lang",
                "en",
                "--sample",
                "--output-dir"
            ])
            .arg(&raw)
            .status()
            .unwrap()
            .success()
    );

    let input = raw.join("wikidata-en.json");
    assert!(input.with_extension("source.json").exists());
    assert!(
        Command::new(bin())
            .args(["lexicon", "prepare", "wikidata", "--input"])
            .arg(&input)
            .args(["--output"])
            .arg(&prepared)
            .args(["--kind", "canonical-map", "--lang", "en"])
            .status()
            .unwrap()
            .success()
    );

    assert!(
        Command::new(bin())
            .args(["lexicon", "build-plugin"])
            .arg(&prepared)
            .args(["--format", "json", "--output"])
            .arg(&plugin)
            .status()
            .unwrap()
            .success()
    );

    let output = Command::new(bin())
        .args(["lexicon", "inspect-plugin"])
        .arg(&plugin)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("canonical-map"));
    assert!(stdout.contains("entries: "));
}

#[test]
fn prepare_build_and_inspect_fst_plugin() {
    let dir = unique_temp_dir("textcase-cli-fst");
    let raw = dir.join("raw");
    let prepared = dir.join("prepared.json");
    let plugin = dir.join("plugin.tclx");
    fs::create_dir_all(&raw).unwrap();

    assert!(
        Command::new(bin())
            .args([
                "lexicon",
                "fetch",
                "gnd",
                "--lang",
                "de",
                "--sample",
                "--output-dir",
            ])
            .arg(&raw)
            .status()
            .unwrap()
            .success()
    );

    let input = raw.join("gnd-de.json");
    assert!(input.with_extension("source.json").exists());
    assert!(
        Command::new(bin())
            .args(["lexicon", "prepare", "gnd", "--input"])
            .arg(&input)
            .args(["--output"])
            .arg(&prepared)
            .args(["--kind", "canonical-map", "--lang", "de"])
            .status()
            .unwrap()
            .success()
    );

    assert!(
        Command::new(bin())
            .args(["lexicon", "build-plugin"])
            .arg(&prepared)
            .args(["--format", "fst", "--output"])
            .arg(&plugin)
            .status()
            .unwrap()
            .success()
    );

    let output = Command::new(bin())
        .args(["lexicon", "inspect-plugin"])
        .arg(&plugin)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("canonical-map"));
    assert!(stdout.contains("entries: "));
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_textcase")
}

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{nanos}"))
}
