use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::sources::SourceId;

#[derive(Debug, Parser)]
#[command(
    name = "textcase",
    about = "Prepare, build, and inspect textcase lexicon plugins"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Lexicon(LexiconArgs),
}

#[derive(Debug, Args)]
pub struct LexiconArgs {
    #[command(subcommand)]
    pub command: LexiconCommand,
}

#[derive(Debug, Subcommand)]
pub enum LexiconCommand {
    /// List supported data sources.
    ListSources,
    /// Show licensing and workflow guidance for a source.
    ShowLicense {
        #[arg(value_enum)]
        source: SourceId,
    },
    /// Fetch source data from a production upstream source.
    Fetch(FetchArgs),
    /// Convert raw source input into a prepared lexicon file.
    Prepare(PrepareArgs),
    /// Build a JSON or FST plugin from a prepared lexicon file.
    BuildPlugin(BuildPluginArgs),
    /// Inspect a JSON or FST plugin file.
    InspectPlugin { path: PathBuf },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PluginFormat {
    Json,
    Fst,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum PreparedKindArg {
    WordSet,
    CanonicalMap,
    MultiwordMap,
    RankedCandidates,
    ProtectedForms,
}

impl From<PreparedKindArg> for textcase::lexicon::PreparedKind {
    fn from(value: PreparedKindArg) -> Self {
        match value {
            PreparedKindArg::WordSet => Self::WordSet,
            PreparedKindArg::CanonicalMap => Self::CanonicalMap,
            PreparedKindArg::MultiwordMap => Self::MultiwordMap,
            PreparedKindArg::RankedCandidates => Self::RankedCandidates,
            PreparedKindArg::ProtectedForms => Self::ProtectedForms,
        }
    }
}

#[derive(Debug, Args)]
pub struct FetchArgs {
    #[arg(value_enum)]
    pub source: SourceId,
    #[arg(long)]
    pub lang: Option<String>,
    #[arg(long)]
    pub country: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long, hide = true)]
    pub sample: bool,
    #[arg(long, default_value = "data/raw")]
    pub output_dir: PathBuf,
    #[arg(long)]
    pub acknowledge_cc_by_sa: bool,
    #[arg(long)]
    pub acknowledge_share_alike: bool,
    #[arg(long)]
    pub acknowledge_odbl: bool,
}

#[derive(Debug, Args)]
pub struct PrepareArgs {
    #[arg(value_enum)]
    pub source: SourceId,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, value_enum)]
    pub kind: PreparedKindArg,
    #[arg(long)]
    pub lang: Option<String>,
    #[arg(long)]
    pub acknowledge_cc_by_sa: bool,
    #[arg(long)]
    pub acknowledge_share_alike: bool,
    #[arg(long)]
    pub acknowledge_odbl: bool,
}

#[derive(Debug, Args)]
pub struct BuildPluginArgs {
    pub input: PathBuf,
    #[arg(long, value_enum)]
    pub format: PluginFormat,
    #[arg(long)]
    pub output: PathBuf,
}
