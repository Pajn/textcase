use std::fs;

use crate::{
    cli::PrepareArgs,
    fsutil::ensure_parent_dir,
    sources::{prepare_source, require_acknowledgement},
};

pub fn run(args: PrepareArgs) -> Result<String, Box<dyn std::error::Error>> {
    require_acknowledgement(
        args.source,
        args.acknowledge_cc_by_sa,
        args.acknowledge_share_alike,
        args.acknowledge_odbl,
    )?;

    let bytes = fs::read(&args.input)?;
    let prepared = prepare_source(args.source, &bytes, args.kind.into(), args.lang.as_deref())?;
    ensure_parent_dir(&args.output)?;
    fs::write(&args.output, serde_json::to_vec_pretty(&prepared)?)?;
    Ok(format!(
        "wrote prepared lexicon to {}",
        args.output.display()
    ))
}
