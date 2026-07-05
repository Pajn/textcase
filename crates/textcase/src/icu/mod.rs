mod casing;
mod segmentation;

pub use casing::{
    capitalize_word_locale, lowercase_locale, primary_language, titlecase_word_locale,
    uppercase_first_grapheme,
};
pub use segmentation::{first_grapheme, split_word_boundaries};
