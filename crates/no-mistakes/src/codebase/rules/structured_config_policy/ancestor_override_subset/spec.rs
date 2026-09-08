use crate::codebase::rules::structured_config_policy::value_at_key;
use serde_yaml::Value;
use std::path::Path;

pub(super) const MAX_EXTENDS_DEPTH: usize = 64;

pub(super) fn is_package_specifier(spec: &str) -> bool {
    !spec.starts_with('.') && !is_portable_absolute(spec)
}

pub(super) fn is_valid_extends_spec(spec: &str) -> bool {
    if spec.trim().is_empty() || is_portable_absolute(spec) {
        return false;
    }
    is_package_specifier(spec) || spec.starts_with("./") || spec.starts_with("../")
}

pub(super) fn extends_specs<'a>(value: &'a Value, key: &str) -> Result<Vec<&'a str>, &'static str> {
    let Some(extends) = value_at_key(value, key) else {
        return Ok(Vec::new());
    };
    let specs = match extends {
        Value::String(spec) => vec![spec.as_str()],
        Value::Sequence(specs) => specs
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()
            .ok_or("must be a string or an array of strings")?,
        _ => return Err("must be a string or an array of strings"),
    };
    if specs.iter().all(|spec| is_valid_extends_spec(spec)) {
        Ok(specs)
    } else {
        Err("must be a non-empty portable relative path or package specifier")
    }
}

fn is_portable_absolute(spec: &str) -> bool {
    Path::new(spec).is_absolute()
        || spec.starts_with(['/', '\\'])
        || spec.as_bytes().get(1) == Some(&b':')
}
