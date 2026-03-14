//! `textcase` provides conservative sentence and title recasing for Latin-script
//! languages with optional lexicon plugins for proper-noun restoration.

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
