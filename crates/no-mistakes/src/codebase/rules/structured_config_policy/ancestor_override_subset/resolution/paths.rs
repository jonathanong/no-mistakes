use serde_yaml::Value;

pub(super) fn extends(value: &Value, key: &str) -> Result<Vec<String>, String> {
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

pub(super) fn local_specifier(specifier: &str) -> Result<Option<String>, String> {
    if specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier.starts_with(".\\")
        || specifier.starts_with("..\\")
    {
        return Ok(Some(specifier.replace('\\', "/")));
    }
    if is_absolute_or_drive_path(specifier) {
        return Err(format!(
            "`extends` reference is outside the repository root: {specifier}"
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

#[cfg(test)]
mod tests {
    use super::{is_absolute_or_drive_path, local_specifier};

    #[test]
    fn classifies_posix_windows_and_package_extends() {
        assert_eq!(
            local_specifier("./base.json").unwrap(),
            Some("./base.json".to_string())
        );
        assert_eq!(
            local_specifier("..\\base.json").unwrap(),
            Some("../base.json".to_string())
        );
        assert_eq!(local_specifier("@scope/config").unwrap(), None);
        for path in [
            "/tmp/base.json",
            "C:\\outside.json",
            "\\\\server\\share\\base.json",
        ] {
            assert!(is_absolute_or_drive_path(path), "{path}");
            assert!(local_specifier(path).is_err(), "{path}");
        }
    }
}
