use super::package_glob::PackageGlob;
use super::{push, validation, FileKind, Issue, Options};
use std::collections::HashSet;

pub(super) struct Registry<'a> {
    pub(super) permanent: HashSet<&'a str>,
    pub(super) temporary: HashSet<String>,
    lock_names: HashSet<&'a str>,
    complete: bool,
}

impl<'a> Registry<'a> {
    pub(super) fn new(
        opts: &'a Options,
        active_names: &HashSet<&'a str>,
        lock_names: HashSet<&'a str>,
        complete: bool,
        issues: &mut Vec<Issue>,
    ) -> Self {
        let mut permanent = HashSet::new();
        let mut has_patterns = false;
        for entry in &opts.permanent_packages {
            let Some(pattern) = entry.pattern.as_deref() else {
                if !entry.name.trim().is_empty() {
                    permanent.insert(entry.name.as_str());
                }
                continue;
            };
            if !entry.name.trim().is_empty() {
                push(
                    issues,
                    FileKind::Workspace,
                    pattern,
                    "must use either name or pattern in permanentPackages",
                );
                continue;
            }
            let Some(glob) = PackageGlob::new(pattern) else {
                invalid_glob(issues, FileKind::Workspace, pattern);
                continue;
            };
            has_patterns = true;
            let mut matches = active_names
                .iter()
                .copied()
                .filter(|name| glob.matches(name))
                .peekable();
            if complete && matches.peek().is_none() {
                push(
                    issues,
                    FileKind::Lockfile,
                    pattern,
                    "is registered but matches no package in manifests or the lockfile",
                );
            }
            permanent.extend(matches);
        }
        Self {
            permanent,
            temporary: validation::temporary_selectors(opts, issues),
            lock_names,
            complete: complete || !has_patterns,
        }
    }

    pub(super) fn check_glob(
        &self,
        issues: &mut Vec<Issue>,
        file: FileKind,
        value: &str,
        glob: &PackageGlob,
    ) {
        if let Some(name) = self
            .lock_names
            .iter()
            .filter(|name| !self.permanent.contains(*name) && glob.matches(name))
            .min()
        {
            push(
                issues,
                file,
                value,
                &format!(r#"matches lockfile package "{name}" outside permanentPackages"#),
            );
        } else if self.complete && !self.permanent.iter().any(|name| glob.matches(name)) {
            push(
                issues,
                file,
                value,
                "matches no package in permanentPackages",
            );
        }
    }
}

pub(super) fn invalid_glob(issues: &mut Vec<Issue>, file: FileKind, value: &str) {
    push(
        issues,
        file,
        value,
        "must be a non-empty package-name pattern without a version",
    );
}
