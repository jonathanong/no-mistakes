use serde_yaml::Value;

pub(crate) fn extends(value: &Value, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = value.get(key) else {
        return Ok(Vec::new());
    };
    match value {
        Value::String(path) => Ok(vec![path.clone()]),
        Value::Sequence(paths) => paths
            .iter()
            .map(|path| match path {
                Value::String(path) => Ok(path.clone()),
                _ => Err(format!("`{key}` must be a string or an array of strings")),
            })
            .collect(),
        _ => Err(format!("`{key}` must be a string or an array of strings")),
    }
}

pub(crate) fn local_specifier(specifier: &str, key: &str) -> Result<Option<String>, String> {
    if specifier.trim().is_empty() {
        return Err(format!("`{key}` reference must not be blank"));
    }
    if (specifier.starts_with('.') && !specifier.starts_with(".."))
        || specifier.starts_with("../")
        || specifier.starts_with("..\\")
    {
        return Ok(Some(specifier.replace('\\', "/")));
    }
    if is_absolute_or_drive_path(specifier) || is_drive_relative_path(specifier) {
        return Err(format!(
            "`{key}` reference is outside the repository root: {specifier}"
        ));
    }
    Ok(None)
}

fn is_absolute_or_drive_path(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    specifier.starts_with('/')
        || specifier.starts_with('\\')
        || specifier.starts_with("//")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
}

fn is_drive_relative_path(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
