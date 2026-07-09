use textcase::{CaseMode, CaseOptions, GermanMode, convert};

fn main() {
    let mut options = CaseOptions::for_locale("de");
    options.mode = CaseMode::Sentence;
    options.german_mode = GermanMode::Balanced;
    println!("{}", convert("ich mag die wissenschaft", &options));
}
