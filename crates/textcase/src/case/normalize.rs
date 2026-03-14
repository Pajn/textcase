use crate::{config::SubtitleSeparatorStyle, util::collapse_whitespace};

pub fn normalize_whitespace(input: &str) -> String {
    collapse_whitespace(input)
}

pub fn normalize_subtitle_separators(input: &str, style: SubtitleSeparatorStyle) -> String {
    if matches!(style, SubtitleSeparatorStyle::Preserve) {
        return input.to_string();
    }

    let mut out = input
        .replace(" — ", " <subtitle> ")
        .replace(" – ", " <subtitle> ")
        .replace(" - ", " <subtitle> ")
        .replace(": ", " <subtitle> ")
        .replace(" : ", " <subtitle> ");

    let replacement = match style {
        SubtitleSeparatorStyle::Preserve => unreachable!(),
        SubtitleSeparatorStyle::ColonSpace => ": ",
        SubtitleSeparatorStyle::SpaceDashSpace => " - ",
        SubtitleSeparatorStyle::EmDashSpace => " — ",
    };

    out = out.replace(" <subtitle> ", replacement);
    out
}
