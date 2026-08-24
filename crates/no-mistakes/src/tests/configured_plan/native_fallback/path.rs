use super::slash_path;

pub(super) fn normalize_config_path(path: &str) -> String {
    let mut path = slash_path(path);
    while let Some(rest) = path.strip_prefix("./") {
        path = rest.to_string();
    }
    if path == "." {
        String::new()
    } else {
        path
    }
}
