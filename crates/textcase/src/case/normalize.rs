use crate::config::SubtitleSeparatorStyle;
use crate::tokenize::Token;

/// Collapses each interior whitespace token to a single space (or a newline,
/// when `flatten_lines` is false and the run contained one) and trims the
/// leading and trailing whitespace tokens to empty. Mirrors the old string-level
/// collapse, but only touches token text: each token's `source` range still maps
/// back to the raw bytes.
pub(crate) fn normalize_whitespace_tokens(tokens: &mut [Token], flatten_lines: bool) {
    let is_content = |token: &Token| !token.is_whitespace();
    let first_content = tokens.iter().position(is_content);
    let last_content = tokens.iter().rposition(is_content);
    // No non-whitespace content means the whole input trims away.
    let (Some(first_content), Some(last_content)) = (first_content, last_content) else {
        for token in tokens.iter_mut() {
            token.text.clear();
        }
        return;
    };

    for (index, token) in tokens.iter_mut().enumerate() {
        if !token.is_whitespace() {
            continue;
        }
        if index < first_content || index > last_content {
            token.text.clear();
        } else {
            let replacement = if !flatten_lines && token.text.contains('\n') {
                "\n"
            } else {
                " "
            };
            if token.text != replacement {
                token.text.clear();
                token.text.push_str(replacement);
            }
        }
    }
}

/// Rewrites the flagged subtitle separators to `style`, folding each separator's
/// punctuation and its immediately flanking whitespace into the replacement so
/// the reconstruction reads exactly `<word><style><word>`. Returns the
/// `(first_token, last_token)` index pairs it rewrote, so the analysis path can
/// surface them as transforms; the plain path discards the value. Token `source`
/// ranges are left untouched.
pub(crate) fn normalize_separator_tokens(
    tokens: &mut [Token],
    separators: &[bool],
    style: SubtitleSeparatorStyle,
) -> Vec<(usize, usize)> {
    let replacement = match style {
        SubtitleSeparatorStyle::Preserve => return Vec::new(),
        SubtitleSeparatorStyle::ColonSpace => ": ",
        SubtitleSeparatorStyle::SpaceDashSpace => " - ",
        SubtitleSeparatorStyle::EmDashSpace => " — ",
    };

    let mut rewrites = Vec::new();
    for index in 0..tokens.len() {
        if !separators.get(index).copied().unwrap_or(false) {
            continue;
        }
        // Absorb the single space the whitespace normalizer left on each side, if
        // present; an em-dash may sit directly between words with none.
        let lead = index > 0 && tokens[index - 1].is_whitespace();
        let trail = tokens.get(index + 1).is_some_and(Token::is_whitespace);
        let first = if lead { index - 1 } else { index };
        let last = if trail { index + 1 } else { index };

        tokens[index].text.clear();
        tokens[index].text.push_str(replacement);
        if lead {
            tokens[index - 1].text.clear();
        }
        if trail {
            tokens[index + 1].text.clear();
        }
        rewrites.push((first, last));
    }
    rewrites
}
