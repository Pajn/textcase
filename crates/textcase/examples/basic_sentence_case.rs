use textcase::sentence_case;

fn main() {
    let value = sentence_case("the rise of github in berlin", "en");
    println!("{value}");
}
