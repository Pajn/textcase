// The README is the crate-level documentation, so its examples are compiled
// and run as doctests and cannot silently go stale.
#![doc = include_str!("../README.md")]

pub mod case;
pub mod config;
pub mod error;
pub mod icu;
pub mod lang;
pub mod lexicon;
pub mod plugin;
pub mod tokenize;
pub mod util;

pub use case::{convert, sentence_case, sentence_case_title};
pub use config::{CaseMode, CaseOptions, GermanMode, SubtitleSeparatorStyle};
pub use error::{Error, Result};
pub use lexicon::{Candidate, LexiconProvider, PluginSet};
