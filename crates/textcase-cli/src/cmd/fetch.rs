use std::{fs, path::PathBuf};

use reqwest::blocking::Client;

use crate::{
    cli::FetchArgs,
    fsutil::ensure_parent_dir,
    sources::{descriptor, require_acknowledgement, sample_payload, suggested_output_name},
};

pub fn run(args: FetchArgs) -> Result<String, Box<dyn std::error::Error>> {
    require_acknowledgement(
        args.source,
        args.acknowledge_cc_by_sa,
        args.acknowledge_share_alike,
        args.acknowledge_odbl,
    )?;

    let path = output_path(
        &args.output_dir,
        suggested_output_name(
            args.source,
            args.lang.as_deref(),
            args.country.as_deref(),
            args.region.as_deref(),
        ),
    );
    ensure_parent_dir(&path)?;
    let bytes = if let Some(url) = &args.url {
        Client::new()
            .get(url)
            .send()?
            .error_for_status()?
            .bytes()?
            .to_vec()
    } else {
        sample_payload(
            args.source,
            args.lang.as_deref(),
            args.country.as_deref(),
            args.region.as_deref(),
        )
    };
    fs::write(&path, bytes)?;

    let descriptor = descriptor(args.source);
    Ok(format!(
        "wrote {} data to {}",
        descriptor.display_name,
        path.display()
    ))
}

fn output_path(output_dir: &std::path::Path, filename: String) -> PathBuf {
    output_dir.join(filename)
}
