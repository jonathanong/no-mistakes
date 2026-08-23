use super::super::BaseDir;

pub(crate) struct Extracted {
    pub(crate) field: String,
    pub(crate) value: String,
    pub(crate) allow_globs: bool,
    pub(crate) base_dir: BaseDir,
}

pub(super) fn is_optional_glob(value: &str) -> bool {
    value.starts_with('!')
        || value
            .split('/')
            .any(|part| part == "node_modules" || part == ".git")
}
