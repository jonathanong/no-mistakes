use super::*;
use crate::codebase::check_outcome::CheckInputFailure;

impl traversal::Outcome {
    pub(super) fn record_source_failure(&mut self, file: &Path, facts: &dyn TsFactLookup) {
        let message = facts.get_ts_facts(file).map_or_else(
            || format!("missing prepared source facts for {}", file.display()),
            |source| {
                source
                    .operational_error
                    .as_deref()
                    .or(source.parse_error.as_deref())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        format!("failed to parse prepared source {}", file.display())
                    })
            },
        );
        self.source_failures.insert(file.to_path_buf(), message);
    }
}

pub(super) fn require_sources(root: &Path, outcome: &traversal::Outcome) -> Result<()> {
    if let Some((file, message)) = outcome.source_failures.first_key_value() {
        return Err(CheckInputFailure::source(
            file.clone(),
            format!(
                "source failure in {}: {message}",
                relative_slash_path(root, file)
            ),
        )
        .into());
    }
    Ok(())
}
