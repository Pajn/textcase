use unicode_segmentation::UnicodeSegmentation;

pub fn split_word_boundaries(input: &str) -> Vec<&str> {
    input.split_word_bounds().collect()
}

pub fn first_grapheme(input: &str) -> Option<&str> {
    input.graphemes(true).next()
}
