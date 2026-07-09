use textcase::{CaseMode, CaseOptions, SubtitleSeparatorStyle, convert};

fn main() {
    let mut options = CaseOptions::for_locale("en");
    options.mode = CaseMode::SentenceTitle;
    options.subtitle_separator_style = SubtitleSeparatorStyle::ColonSpace;
    println!("{}", convert("cities - the rise of berlin", &options));
}
