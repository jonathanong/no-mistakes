pub(super) fn declare_module_specifiers(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut index = 0;
    while let Some(offset) = source[index..].find("declare") {
        let at = index + offset;
        let boundary = at == 0 || !is_ident_byte(source.as_bytes()[at - 1]);
        let after = at + "declare".len();
        if boundary {
            if let Some(name) = module_name_follows(&source[after..]) {
                found.push(name.to_owned());
            }
        }
        index = after;
    }
    found.sort();
    found.dedup();
    found
}

fn module_name_follows(after_declare: &str) -> Option<&str> {
    let trimmed = after_declare.trim_start();
    if trimmed.len() == after_declare.len() {
        return None;
    }
    let after_keyword = trimmed.strip_prefix("module")?;
    let quoted = after_keyword.trim_start();
    let quote = quoted.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let rest = &quoted[1..];
    let name = &rest[..rest.find(quote)?];
    (!name.is_empty()).then_some(name)
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
