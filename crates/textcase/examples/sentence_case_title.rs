use textcase::{CaseMode, CaseOptions, SubtitleSeparatorStyle, convert};

fn main() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    let value = convert("a tale of berlin - from streets to skylines", &options);
    println!("{value}");
}
