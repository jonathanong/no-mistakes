use super::{slash_normalize, AllowList, RuleFinding};

impl AllowList {
    pub fn apply(self, catalog_path: &str, findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
        self.apply_with(catalog_path, findings, |finding| finding, |finding| finding)
    }

    pub(crate) fn apply_located(
        self,
        catalog_path: &str,
        findings: crate::codebase::rules::PostgresFindings,
    ) -> crate::codebase::rules::PostgresFindings {
        crate::codebase::rules::PostgresFindings(self.apply_with(
            catalog_path,
            findings.0,
            |located| &located.finding,
            crate::codebase::rules::LocatedRuleFinding::from,
        ))
    }

    fn apply_with<T>(
        self,
        catalog_path: &str,
        mut findings: Vec<T>,
        finding: impl Fn(&T) -> &RuleFinding,
        wrap: impl Fn(RuleFinding) -> T,
    ) -> Vec<T> {
        let path = slash_normalize(catalog_path);
        let used = crate::codebase::rules::allow_audit::AllowUsage::new(self.entries.len());
        findings.retain(|finding_value| {
            let Some(target) = finding(finding_value).target.as_deref() else {
                return true;
            };
            !used.suppress(self.entries.iter().map(|entry| entry.object == target))
        });
        for index in used.unused() {
            let entry = &self.entries[index];
            findings.push(wrap(RuleFinding {
                causes: Vec::new(),
                rule: self.rule_id.clone(),
                file: path.clone(),
                line: 1,
                message: format!(
                    "{path}: stale {} allow entry: {}",
                    self.rule_id, entry.object
                ),
                import: None,
                target: Some(entry.object.clone()),
            }));
        }
        findings
    }
}
