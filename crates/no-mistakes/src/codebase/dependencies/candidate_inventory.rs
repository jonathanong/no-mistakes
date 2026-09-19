use crate::codebase::rules::path_filter::GlobMatcher;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct CandidateBounds {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
}

impl CandidateBounds {
    pub(crate) fn from_args(args: &TraverseArgs) -> Self {
        Self {
            include: canonicalize_globs(&args.candidate_include),
            exclude: canonicalize_globs(&args.candidate_exclude),
        }
    }
}

fn canonicalize_globs(globs: &[String]) -> Vec<String> {
    let mut globs = globs.to_vec();
    globs.sort();
    globs.dedup();
    globs
}

pub(crate) fn validate_candidate_bounds(args: &TraverseArgs, direction: Direction) -> Result<()> {
    if args.projection.is_paths() && !matches!(direction, Direction::Deps) {
        bail!("projection paths is only valid for dependencies");
    }
    if !args.has_candidate_bounds() {
        return Ok(());
    }
    if !matches!(direction, Direction::Deps) {
        bail!("candidateInclude and candidateExclude are only valid for dependencies");
    }
    if args.include_symbols {
        bail!("candidateInclude and candidateExclude cannot be combined with includeSymbols");
    }
    if !relationships_are_import_only(&args.relationships) {
        bail!("candidateInclude and candidateExclude require import-only relationships");
    }
    Ok(())
}

pub(crate) struct CandidateInventory {
    include: GlobMatcher,
    exclude: GlobMatcher,
}

impl CandidateInventory {
    pub(crate) fn new(include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            include: GlobMatcher::new(include, "candidateInclude")?,
            exclude: GlobMatcher::new(exclude, "candidateExclude")?,
        })
    }

    pub(crate) fn matches(&self, root: &Path, path: &Path) -> bool {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        let included = self.include.is_empty() || self.include.is_match(&rel);
        included && (self.exclude.is_empty() || !self.exclude.is_match(&rel))
    }

    pub(crate) fn filter_paths(
        &self,
        root: &Path,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> Vec<PathBuf> {
        paths
            .into_iter()
            .filter(|path| self.matches(root, path))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BoundedImportKey {
    bounds: CandidateBounds,
    relationships: Vec<String>,
}

impl BoundedImportKey {
    pub(crate) fn from_args(args: &TraverseArgs) -> Self {
        Self {
            bounds: CandidateBounds::from_args(args),
            relationships: bounded_relationship_key(args),
        }
    }
}

pub(crate) fn bounded_relationship_key(args: &TraverseArgs) -> Vec<String> {
    let mut relationships = args
        .relationships
        .iter()
        .map(|relationship| relationship.as_str().to_string())
        .collect::<Vec<_>>();
    relationships.sort();
    relationships.dedup();
    relationships
}
