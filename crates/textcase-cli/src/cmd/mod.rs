mod build_plugin;
mod fetch;
mod inspect_plugin;
mod list_sources;
mod prepare;
mod show_license;

use crate::cli::{Cli, Commands, LexiconCommand};

pub fn run(cli: Cli) -> Result<String, Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Lexicon(lexicon) => match lexicon.command {
            LexiconCommand::ListSources => list_sources::run(),
            LexiconCommand::ShowLicense { source } => show_license::run(source),
            LexiconCommand::Fetch(args) => fetch::run(args),
            LexiconCommand::Prepare(args) => prepare::run(args),
            LexiconCommand::BuildPlugin(args) => build_plugin::run(args),
            LexiconCommand::InspectPlugin { path } => inspect_plugin::run(path),
        },
    }
}
