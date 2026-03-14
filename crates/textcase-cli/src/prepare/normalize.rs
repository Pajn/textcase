use crate::sources::SourceRecord;

pub fn normalized_aliases(record: &SourceRecord) -> Vec<String> {
    let mut aliases = vec![record.canonical.to_lowercase()];
    aliases.extend(record.aliases.iter().map(|alias| alias.to_lowercase()));
    aliases.sort();
    aliases.dedup();
    aliases
}
