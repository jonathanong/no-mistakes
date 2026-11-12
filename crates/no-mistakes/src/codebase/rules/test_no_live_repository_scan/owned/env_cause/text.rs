use super::EnvCause;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

impl EnvCause {
    pub(in super::super::super) fn message(&self, root: &Path, callee: &str) -> String {
        let name = &self.hint.name;
        let place = format!("{}:{}", relative_slash_path(root, &self.file), self.line);
        let fallback = self
            .hint
            .fallback
            .as_ref()
            .map(|value| {
                format!(
                    ", with literal fallback {}",
                    serde_json::to_string(value).unwrap_or_default()
                )
            })
            .unwrap_or_default();
        let variable = serde_json::to_string(name).unwrap_or_default();
        let (problem, fix) = if self.concrete {
            (format!("{place} {}", self.reason), format!("at {place}, use vi.stubEnv({variable}, fixturePath) with a fixture or owned mkdtemp root before the call or dynamic import; do not clear it until the scan has finished"))
        } else {
            ("a concrete test or setup entry cannot be identified, so environment ownership cannot be established".to_string(), format!("make the test-to-scan call or import route explicit, then use vi.stubEnv({variable}, fixturePath) with a fixture or owned mkdtemp root in that test or setup before the scan"))
        };
        format!("unresolved root: `{callee}` enumerates a root derived from `process.env.{name}`{fallback}, but {problem}. Fix: {fix}. For a captured alias, read the environment at the scan or pass the owned root directly; keep derived paths inside that owned root. As a last option, at the enumeration add `no-mistakes-disable-next-line test-no-live-repository-scan` with a reason.")
    }
}
