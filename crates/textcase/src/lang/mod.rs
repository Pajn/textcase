pub mod german;
mod profile;

mod azerbaijani;
mod danish;
mod dutch;
mod english;
mod finnish;
mod french;
mod italian;
mod lithuanian;
mod norwegian;
mod portuguese;
mod spanish;
mod swedish;
mod turkish;

pub(crate) use english::always_capitalized as english_always_capitalized;
pub use profile::LanguageProfile;

pub fn profile_for_locale(locale: &str) -> LanguageProfile {
    let language = locale.split(['-', '_']).next().unwrap_or(locale);
    match language {
        "fr" => french::profile(),
        "es" => spanish::profile(),
        "pt" => portuguese::profile(),
        "it" => italian::profile(),
        "nl" => dutch::profile(),
        "sv" => swedish::profile(),
        "da" => danish::profile(),
        "no" | "nb" | "nn" => norwegian::profile(),
        "fi" => finnish::profile(),
        "tr" => turkish::profile(),
        "az" => azerbaijani::profile(),
        "lt" => lithuanian::profile(),
        "de" => german::profile(),
        "en" => english::profile(),
        // An unknown language gets the neutral profile: applying English stop
        // words and particles to it would be a guess, not a default.
        _ => LanguageProfile::neutral(),
    }
}
