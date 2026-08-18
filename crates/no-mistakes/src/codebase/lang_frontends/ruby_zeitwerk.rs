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
                && path.components().any(|part| part.as_os_str() == "app")
                && roots.iter().any(|root| path.starts_with(root))
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
                candidate.starts_with(owner)
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

fn zeitwerk_rel(name: &str) -> String {
    let mut parts = Vec::new();
    for segment in name.split("::") {
        parts.push(underscore(segment));
    }
    format!("/{}.rb", parts.join("/"))
}

fn underscore(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
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
