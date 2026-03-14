use std::fs;

use textcase::lexicon::PreparedLexicon;

use crate::{
    checksum::file_checksum,
    cli::{BuildPluginArgs, PluginFormat},
    fsutil::ensure_parent_dir,
    manifest::OutputManifest,
    prepare::fst_build::build_fst_plugin,
};

pub fn run(args: BuildPluginArgs) -> Result<String, Box<dyn std::error::Error>> {
    let prepared: PreparedLexicon = serde_json::from_slice(&fs::read(&args.input)?)?;
    ensure_parent_dir(&args.output)?;

    match args.format {
        PluginFormat::Json => {
            let plugin = prepared.to_plugin_schema();
            fs::write(&args.output, serde_json::to_vec_pretty(&plugin)?)?;
        }
        PluginFormat::Fst => {
            build_fst_plugin(&prepared, &args.output)?;
        }
    }

    let manifest = OutputManifest {
        input: args.input.display().to_string(),
        output: args.output.display().to_string(),
        format: match args.format {
            PluginFormat::Json => "json".to_string(),
            PluginFormat::Fst => "fst".to_string(),
        },
        checksum: file_checksum(&args.output)?,
    };

    Ok(serde_json::to_string_pretty(&manifest)?)
}
