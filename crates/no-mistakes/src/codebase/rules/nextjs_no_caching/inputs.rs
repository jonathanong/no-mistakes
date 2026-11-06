use super::*;
use crate::codebase::config::InferredRoots;
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::config::v2::schema::RuleDef;

pub(crate) struct SourceInputs {
    roots: Vec<PathBuf>,
    filter: RulePathFilter,
}

impl SourceInputs {
    pub(crate) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        inferred: &mut InferredRoots,
    ) -> Result<Self> {
        let roots = super::super::target_roots_with_inferred(root, config, rule, inferred);
        let filter = RulePathFilter::new_with_inferred(root, config, rule, inferred)?;
        Ok(Self { roots, filter })
    }

    pub(crate) fn contains(&self, path: &Path) -> bool {
        is_indexable(path) && self.roots.iter().any(|root| path.starts_with(root)) && {
            record_aggregate_scope_check!();
            self.filter.is_match(path)
        }
    }
}
