use super::super::facts::LangFactMap;
use std::path::PathBuf;

pub(super) fn attach_zeitwerk_refs(
    facts: &mut LangFactMap,
    roots: &[PathBuf],
    all_files: &[PathBuf],
) {
    let app_files: Vec<&PathBuf> = all_files
        .iter()
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("rb")
                && roots.iter().any(|root| under_app(root, path))
        })
        .collect();
    let refs: Vec<(PathBuf, Vec<String>)> = facts
        .files
        .values()
        .map(|file| (file.path.clone(), file.references.clone()))
        .collect();
    for (path, references) in refs {
        let Some(owner) = roots
            .iter()
            .filter(|root| path.starts_with(root))
            .max_by_key(|root| root.components().count())
        else {
            continue;
        };
        for reference in references {
            let rel = zeitwerk_rel(&reference);
            let Some(target) = app_files.iter().copied().find(|candidate| {
                under_app(owner, candidate)
                    && candidate
                        .to_string_lossy()
                        .replace('\\', "/")
                        .ends_with(&rel)
            }) else {
                continue;
            };
            facts
                .declarations
                .entry(reference)
                .or_default()
                .insert(target.clone());
        }
    }
}

fn under_app(root: &std::path::Path, path: &std::path::Path) -> bool {
    path.strip_prefix(root)
        .ok()
        .and_then(|rel| rel.components().next())
        .is_some_and(|part| part.as_os_str() == "app")
}

fn zeitwerk_rel(name: &str) -> String {
    let mut parts = Vec::new();
    for segment in name.split("::") {
        parts.push(underscore(segment));
    }
    format!("/{}.rb", parts.join("/"))
}

fn underscore(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let prev_lower = index > 0 && chars[index - 1].is_lowercase();
            let next_lower = chars.get(index + 1).is_some_and(|next| next.is_lowercase());
            if index > 0 && (prev_lower || next_lower) {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
#[path = "ruby_zeitwerk_tests.rs"]
mod tests;
