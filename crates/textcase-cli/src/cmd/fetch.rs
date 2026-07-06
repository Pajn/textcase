use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};

use crate::{
    cli::FetchArgs,
    fsutil::ensure_parent_dir,
    manifest::{FetchedSourceManifest, write_source_manifest},
    sources::{
        built_in_fetch_plan, descriptor, normalize_download, require_acknowledgement,
        requires_normalization, sample_payload, suggested_output_name, validate_source_bytes,
    },
};

pub fn run(args: FetchArgs) -> Result<String, Box<dyn std::error::Error>> {
    require_acknowledgement(
        args.source,
        args.acknowledge_cc_by_sa,
        args.acknowledge_share_alike,
        args.acknowledge_odbl,
    )?;

    let plan = resolve_plan(&args)?;
    let path = args
        .output_dir
        .join(suggested_output_name(args.source, &plan.output_suffix));
    ensure_parent_dir(&path)?;

    match plan.payload {
        // Samples are generated in memory and are small; write them directly.
        Payload::Sample(bytes) => fs::write(&path, &bytes)?,
        Payload::Urls(urls) => {
            let client = http_client()?;
            // Stream each response body to a temporary part file next to the
            // output so the payload is never held in memory during transfer.
            let parts = stream_parts(&client, &urls, &path)?;
            let outcome = finalize(args.source, &parts, &path);
            for part in &parts {
                let _ = fs::remove_file(part);
            }
            outcome?;
        }
    }

    write_source_manifest(
        &path,
        &FetchedSourceManifest {
            source: args.source.to_string(),
            source_url: plan.source_url,
            version: plan.version,
            sample: plan.sample,
        },
    )?;

    let descriptor = descriptor(args.source);
    Ok(format!(
        "wrote {} data to {}",
        descriptor.display_name,
        path.display()
    ))
}

enum Payload {
    Sample(Vec<u8>),
    Urls(Vec<String>),
}

struct ResolvedPlan {
    payload: Payload,
    source_url: String,
    version: String,
    output_suffix: String,
    sample: bool,
}

fn resolve_plan(args: &FetchArgs) -> Result<ResolvedPlan, Box<dyn std::error::Error>> {
    if args.sample {
        Ok(ResolvedPlan {
            payload: Payload::Sample(sample_payload(
                args.source,
                args.lang.as_deref(),
                args.country.as_deref(),
                args.region.as_deref(),
            )),
            source_url: format!("sample://textcase/{}", args.source),
            version: "deterministic-sample".to_string(),
            output_suffix: suffix_or(args, "sample"),
            sample: true,
        })
    } else if let Some(url) = &args.url {
        Ok(ResolvedPlan {
            payload: Payload::Urls(vec![url.clone()]),
            source_url: url.clone(),
            version: "user-supplied".to_string(),
            output_suffix: suffix_or(args, "custom"),
            sample: false,
        })
    } else {
        let plan = built_in_fetch_plan(
            args.source,
            args.lang.as_deref(),
            args.country.as_deref(),
            args.region.as_deref(),
        )?;
        Ok(ResolvedPlan {
            payload: Payload::Urls(plan.urls),
            source_url: plan.source_url,
            version: plan.version,
            output_suffix: plan.output_suffix,
            sample: false,
        })
    }
}

fn suffix_or(args: &FetchArgs, fallback: &str) -> String {
    args.lang
        .as_deref()
        .or(args.country.as_deref())
        .or(args.region.as_deref())
        .unwrap_or(fallback)
        .to_lowercase()
}

/// Streams each URL's response body to a `.partN` file beside the output.
fn stream_parts(
    client: &Client,
    urls: &[String],
    path: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut parts = Vec::with_capacity(urls.len());
    for (index, url) in urls.iter().enumerate() {
        let part = part_path(path, index);
        if let Err(error) = stream_to_file(client, url, &part) {
            // Best-effort cleanup of anything already streamed this call.
            let _ = fs::remove_file(&part);
            for done in &parts {
                let _ = fs::remove_file(done);
            }
            return Err(error);
        }
        parts.push(part);
    }
    Ok(parts)
}

fn stream_to_file(
    client: &Client,
    url: &str,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = client.get(url).send()?.error_for_status()?;
    let mut file = File::create(dest)?;
    io::copy(&mut response, &mut file)?;
    Ok(())
}

/// `dir/name.ext` -> `dir/name.ext.partN`, keeping the temp file beside the
/// output so the final rename stays on the same filesystem.
fn part_path(path: &Path, index: usize) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".part{index}"));
    path.with_file_name(name)
}

/// Turns the streamed part files into the final output. A single pass-through
/// download is validated and renamed into place; sources that need zip
/// extraction, JSON merge, or concatenation read their parts and normalize.
fn finalize(
    source: crate::sources::SourceId,
    parts: &[PathBuf],
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if requires_normalization(source) {
        let mut downloads = Vec::with_capacity(parts.len());
        for part in parts {
            downloads.push(fs::read(part)?);
        }
        let bytes = normalize_download(source, downloads)?;
        validate_source_bytes(source, &bytes)?;
        fs::write(path, &bytes)?;
    } else {
        // A pass-through source is a single download; more than one part would
        // mean silently dropping data, so enforce the invariant explicitly.
        let [part] = parts else {
            return Err(format!(
                "{source} is a single-payload source but {} parts were fetched",
                parts.len()
            )
            .into());
        };
        // Validate over a memory map rather than reading the whole file onto the
        // heap, so streaming validators (e.g. discogs) never materialize a large
        // payload just to check it. The streamed file itself becomes the output.
        let file = File::open(part)?;
        // Safe: we just wrote this temp file and nothing else touches it before
        // the rename below.
        let mapped = unsafe { Mmap::map(&file)? };
        validate_source_bytes(source, &mapped)?;
        drop(mapped);
        drop(file);
        fs::rename(part, path)?;
    }
    Ok(())
}

fn http_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("textcase-cli/0.1.0 (+https://github.com/github/copilot-cli)"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "application/json, application/*+json, text/plain;q=0.9, */*;q=0.8",
        ),
    );
    // The blocking client applies `timeout` per read/write operation (default
    // 30s), so a stalled transfer already fails rather than hanging forever
    // while a progressing large download is not capped.
    Ok(Client::builder().default_headers(headers).build()?)
}
