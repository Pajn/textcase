use crate::sources::descriptors;

pub fn run() -> Result<String, Box<dyn std::error::Error>> {
    let mut lines = vec!["id\tclass\trecommendation\tpurpose\ttags\tlicense".to_string()];
    for descriptor in descriptors() {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            descriptor.id,
            descriptor.class,
            if descriptor.recommended {
                "recommended"
            } else {
                "optional"
            },
            descriptor.purpose,
            descriptor.domain_tags.join(","),
            descriptor.license_summary,
        ));
    }
    Ok(lines.join("\n"))
}
