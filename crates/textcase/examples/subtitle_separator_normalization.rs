use textcase::{CaseMode, CaseOptions, SubtitleSeparatorStyle, convert};

fn main() {
    let options = CaseOptions {
        locale: "en",
        mode: CaseMode::SentenceTitle,
        subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
        ..CaseOptions::default()
    };
    println!("{}", convert("cities - the rise of berlin", &options));
}
