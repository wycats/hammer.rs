pub fn canonical_field_name(field: &str) -> String {
    format!("--{}", field.chars().map(|c|
        if c == '_' {'-'} else {c}).collect::<String>())
}
