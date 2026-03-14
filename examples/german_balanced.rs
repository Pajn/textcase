use textcase::{convert, CaseMode, CaseOptions, GermanMode};

fn main() {
    let options = CaseOptions {
        locale: "de",
        mode: CaseMode::Sentence,
        german_mode: GermanMode::Balanced,
        ..CaseOptions::default()
    };
    println!("{}", convert("ich mag die wissenschaft", &options));
}
