use super::*;

pub(super) fn audited(file: &str, scopes: &[ScopeSpec]) -> bool {
    !is_typescript_declaration_file(file)
        && scopes.iter().any(|scope| {
            normalized_scope_path(&scope.path).is_some_and(|path| {
                path == "." || file == path || file.starts_with(&format!("{path}/"))
            })
        })
}

pub(super) fn file_extension(rel: &str) -> &str {
    match rel.rfind('.') {
        Some(i) => &rel[i..],
        None => "",
    }
}
