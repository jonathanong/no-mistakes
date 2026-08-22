use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct LangFilesByExt {
    pub py: Vec<PathBuf>,
    pub go: Vec<PathBuf>,
    pub rs: Vec<PathBuf>,
    pub rb: Vec<PathBuf>,
    pub php: Vec<PathBuf>,
    pub java: Vec<PathBuf>,
    pub kt: Vec<PathBuf>,
    pub elixir: Vec<PathBuf>,
    pub yaml: Vec<PathBuf>,
    pub yml: Vec<PathBuf>,
    pub json: Vec<PathBuf>,
}

impl LangFilesByExt {
    pub(crate) fn php_universe(&self) -> Vec<PathBuf> {
        let mut files = self.php.clone();
        files.extend(self.json.iter().cloned());
        files.extend(self.yaml.iter().cloned());
        files.extend(self.yml.iter().cloned());
        files
    }
}

pub(crate) fn partition_files_by_extension(all_files: &[PathBuf]) -> LangFilesByExt {
    let mut parts = LangFilesByExt::default();
    for path in all_files {
        if path.file_name().and_then(|name| name.to_str()) == Some("go.mod") {
            parts.go.push(path.clone());
            continue;
        }
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("py") => parts.py.push(path.clone()),
            Some("go") => parts.go.push(path.clone()),
            Some("rs") => parts.rs.push(path.clone()),
            Some("rb") => parts.rb.push(path.clone()),
            Some("php") => parts.php.push(path.clone()),
            Some("java") => parts.java.push(path.clone()),
            Some("kt") => parts.kt.push(path.clone()),
            Some("ex") | Some("exs") => parts.elixir.push(path.clone()),
            Some("yaml") => parts.yaml.push(path.clone()),
            Some("yml") => parts.yml.push(path.clone()),
            Some("json") => parts.json.push(path.clone()),
            _ => {}
        }
    }
    parts
}

#[cfg(test)]
mod tests;
