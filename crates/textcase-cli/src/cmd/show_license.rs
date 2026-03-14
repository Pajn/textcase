use crate::sources::{SourceId, descriptor};

pub fn run(source: SourceId) -> Result<String, Box<dyn std::error::Error>> {
    let descriptor = descriptor(source);
    let acknowledgement = descriptor.acknowledgement_flag.unwrap_or("none required");

    Ok(format!(
        "{name} ({id})
class: {class}
license: {license_name}
summary: {summary}
plugin kinds: {plugin_kinds}
acknowledgement: {ack}
recommendation: {recommendation}
bundling policy: {bundling}
docs anchor: {anchor}",
        name = descriptor.display_name,
        id = descriptor.id,
        class = descriptor.class,
        license_name = descriptor.license_name,
        summary = descriptor.license_summary,
        plugin_kinds = descriptor
            .plugin_kinds
            .iter()
            .map(|kind| format!("{kind:?}"))
            .collect::<Vec<_>>()
            .join(", "),
        ack = acknowledgement,
        recommendation = if descriptor.recommended {
            "recommended"
        } else {
            "optional"
        },
        bundling = descriptor.bundling_policy,
        anchor = descriptor.docs_anchor,
    ))
}
