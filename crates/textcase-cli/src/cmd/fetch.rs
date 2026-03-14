use std::fs;

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};

use crate::{
    cli::FetchArgs,
    fsutil::ensure_parent_dir,
    manifest::{FetchedSourceManifest, write_source_manifest},
    sources::{
        built_in_fetch_plan, descriptor, normalize_download, require_acknowledgement,
        sample_payload, suggested_output_name, validate_source_bytes,
    },
};

pub fn run(args: FetchArgs) -> Result<String, Box<dyn std::error::Error>> {
    require_acknowledgement(
        args.source,
        args.acknowledge_cc_by_sa,
        args.acknowledge_share_alike,
        args.acknowledge_odbl,
    )?;

    let fetched = if args.sample {
        let suffix = args
            .lang
            .as_deref()
            .or(args.country.as_deref())
            .or(args.region.as_deref())
            .unwrap_or("sample")
            .to_lowercase();
        FetchedPayload {
            bytes: sample_payload(
                args.source,
                args.lang.as_deref(),
                args.country.as_deref(),
                args.region.as_deref(),
            ),
            source_url: format!("sample://textcase/{}", args.source),
            version: "deterministic-sample".to_string(),
            output_suffix: suffix,
            sample: true,
        }
    } else if let Some(url) = &args.url {
        let bytes = http_client()?
            .get(url)
            .send()?
            .error_for_status()?
            .bytes()?
            .to_vec();
        let bytes = normalize_download(args.source, vec![bytes])?;
        validate_source_bytes(args.source, &bytes)?;
        FetchedPayload {
            bytes,
            source_url: url.clone(),
            version: "user-supplied".to_string(),
            output_suffix: args
                .lang
                .as_deref()
                .or(args.country.as_deref())
                .or(args.region.as_deref())
                .unwrap_or("custom")
                .to_lowercase(),
            sample: false,
        }
    } else {
        let plan = built_in_fetch_plan(
            args.source,
            args.lang.as_deref(),
            args.country.as_deref(),
            args.region.as_deref(),
        )?;
        let client = http_client()?;
        let mut downloads = Vec::with_capacity(plan.urls.len());
        for url in &plan.urls {
            downloads.push(
                client
                    .get(url)
                    .send()?
                    .error_for_status()?
                    .bytes()?
                    .to_vec(),
            );
        }
        let bytes = normalize_download(args.source, downloads)?;
        validate_source_bytes(args.source, &bytes)?;
        FetchedPayload {
            bytes,
            source_url: plan.source_url,
            version: plan.version,
            output_suffix: plan.output_suffix,
            sample: false,
        }
    };

    let path = args
        .output_dir
        .join(suggested_output_name(args.source, &fetched.output_suffix));
    ensure_parent_dir(&path)?;
    fs::write(&path, &fetched.bytes)?;
    write_source_manifest(
        &path,
        &FetchedSourceManifest {
            source: args.source.to_string(),
            source_url: fetched.source_url,
            version: fetched.version,
            sample: fetched.sample,
        },
    )?;

    let descriptor = descriptor(args.source);
    Ok(format!(
        "wrote {} data to {}",
        descriptor.display_name,
        path.display()
    ))
}

struct FetchedPayload {
    bytes: Vec<u8>,
    source_url: String,
    version: String,
    output_suffix: String,
    sample: bool,
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
    Ok(Client::builder().default_headers(headers).build()?)
}
