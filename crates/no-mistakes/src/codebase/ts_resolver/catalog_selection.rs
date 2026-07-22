impl TsConfigCatalog {
    fn selection(&self, importing_file: &Path) -> Option<usize> {
        if self.forced {
            return (!self.configs.is_empty()).then_some(0);
        }
        let importing_file = real_path(importing_file).unwrap_or_else(|| normalize_path(importing_file));
        let applicable = self
            .configs
            .iter()
            .enumerate()
            .filter(|(_, config)| config.matcher.owns_source(&importing_file))
            .collect::<Vec<_>>();
        let selected = self.deepest_unique(importing_file.as_path(), &applicable);
        match selected {
            Some(index) if !self.blocked_by_broken_boundary(&importing_file, self.configs[index].matcher.ownership_depth()) => {
                Some(index)
            }
            Some(_) => None,
            None => self.nearest_fallback(&importing_file),
        }
    }

    fn deepest_unique(&self, file: &Path, matches: &[(usize, &CatalogConfig)]) -> Option<usize> {
        let deepest = matches.iter().map(|(_, config)| config.matcher.ownership_depth()).max()?;
        let candidates = matches
            .iter()
            .filter(|(_, config)| config.matcher.ownership_depth() == deepest)
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        (candidates.len() == 1).then_some(candidates[0]).or_else(|| {
            self.ambiguous(file, &candidates);
            None
        })
    }

    fn nearest_fallback(&self, file: &Path) -> Option<usize> {
        let valid = self
            .configs
            .iter()
            .enumerate()
            .filter(|(_, config)| file.starts_with(&config.matcher.real_dir))
            .collect::<Vec<_>>();
        let valid_depth = valid.iter().map(|(_, config)| config.matcher.ownership_depth()).max();
        if valid_depth.is_none_or(|depth| self.blocked_by_broken_boundary(file, depth)) {
            return None;
        }
        self.deepest_unique(file, &valid)
    }

    fn blocked_by_broken_boundary(&self, file: &Path, owner_depth: usize) -> bool {
        self.broken_dirs
            .iter()
            .filter(|dir| file.starts_with(dir))
            .map(|dir| path_depth(dir))
            .max()
            .is_some_and(|broken_depth| broken_depth >= owner_depth)
    }

    fn ambiguous(&self, file: &Path, indexes: &[usize]) {
        let mut candidates = indexes.iter().map(|index| self.configs[*index].path.clone()).collect::<Vec<_>>();
        candidates.sort();
        self.push_diagnostic(TsConfigDiagnostic {
            kind: TsConfigDiagnosticKind::AmbiguousOwnership,
            config: None,
            file: Some(file.to_path_buf()),
            detail: "multiple TypeScript configs claim this source at the same directory depth".to_string(),
            candidates,
        });
    }

    fn push_diagnostic(&self, diagnostic: TsConfigDiagnostic) {
        self.diagnostics
            .lock()
            .expect("tsconfig catalog diagnostics mutex poisoned")
            .insert(diagnostic);
    }
}

impl ConfigMatcher {
    fn all(dir: PathBuf) -> Self {
        let real_dir = real_path(&dir).unwrap_or_else(|| dir.clone());
        Self { dir, real_dir, files: None, includes: None, excludes: Vec::new(), out_dir: None, allow_js: true }
    }

    fn ownership_depth(&self) -> usize {
        path_depth(&self.real_dir)
    }

    fn owns_source(&self, file: &Path) -> bool {
        let lexical = file
            .strip_prefix(&self.real_dir)
            .map(|relative| self.dir.join(relative));
        self.owns(lexical.as_deref().unwrap_or(file))
    }

    fn owns(&self, file: &Path) -> bool {
        if matches!(self.files.as_ref(), Some(files) if files.contains(file)) {
            return true;
        }
        if self.files.is_some()
            || !is_config_source(file, self.allow_js)
            || (!file.starts_with(&self.dir) && self.includes.is_none())
            || self.out_dir.as_ref().is_some_and(|out_dir| file.starts_with(out_dir))
        {
            return false;
        }
        self.includes
            .as_ref()
            .map(|rules| rules.iter().any(|rule| rule.matches(file)))
            .unwrap_or(true)
            && !self.excludes.iter().any(|rule| rule.matches(file))
    }
}

impl GlobRule {
    fn new(base: &Path, value: &str) -> Option<Self> {
        let value = value.replace('\\', "/");
        let value = value.trim_start_matches("./");
        let absolute = Path::new(value).is_absolute();
        let allow_parent = value.split('/').any(|component| component == "..");
        // TypeScript treats `include: ["."]` as the configuration directory,
        // not as a literal entry that excludes every descendant source file.
        let value = if value.is_empty() || value == "." {
            "**/*".to_string()
        } else {
            value.to_string()
        };
        let value = if !value.contains('*')
            && !value.contains('?')
            && !value.contains('[')
            && Path::new(&value).extension().is_none()
        {
            format!("{}/**", value.trim_end_matches('/'))
        } else {
            value
        };
        let glob = GlobBuilder::new(&value).literal_separator(true).build().ok()?;
        Some(Self { base: base.to_path_buf(), matcher: glob.compile_matcher(), allow_parent, absolute })
    }

    fn matches(&self, path: &Path) -> bool {
        if self.absolute {
            return self.matcher.is_match(normalize_path(path));
        }
        relative_path(&self.base, path).is_some_and(|relative| {
            (self.allow_parent || !relative.starts_with("..")) && self.matcher.is_match(relative)
        })
    }
}
