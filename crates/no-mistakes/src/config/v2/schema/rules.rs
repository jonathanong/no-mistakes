/// A configured rule application.
///
/// Unlike ESLint-style rule maps, no-mistakes rules are reusable applications:
/// the same `rule` can be attached to different projects or test groups with
/// different names and options.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleDef {
    #[serde(skip)]
    pub resolved_analysis: Option<super::EffectiveRuleAnalysis>,
    pub name: Option<String>,
    pub rule: String,
    pub message: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub projects: Vec<String>,
    pub tests: RuleTestTargets,
    pub scope: Option<RuleScope>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    #[serde(default = "empty_options")]
    pub options: serde_yaml::Value,
    /// `error` (default) fails `check`; `advisory` reports the findings as
    /// non-failing advisories.
    #[serde(skip_serializing_if = "RuleSeverity::is_error")]
    pub severity: RuleSeverity,
}

impl RuleDef {
    /// Every key a rule entry accepts. Config loading rejects anything else.
    pub const SUPPORTED_KEYS: &'static [&'static str] = &[
        "name", "rule", "message", "enabled", "projects", "tests", "scope", "include", "exclude",
        "options", "severity",
    ];
}

/// Staged-rollout severity of a rule application.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    /// Findings fail the run (exit 1). This is the default.
    #[default]
    Error,
    /// Findings are still reported, but as non-failing advisories.
    Advisory,
}

impl RuleSeverity {
    fn is_error(&self) -> bool {
        *self == Self::Error
    }
}

impl Default for RuleDef {
    fn default() -> Self {
        Self {
            resolved_analysis: None,
            name: None,
            rule: String::new(),
            message: None,
            enabled: true,
            projects: Vec::new(),
            tests: RuleTestTargets::default(),
            scope: None,
            include: Vec::new(),
            exclude: Vec::new(),
            options: empty_options(),
            severity: RuleSeverity::Error,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleTestTargets {
    pub dotnet: Vec<String>,
    pub vitest: Vec<String>,
    pub playwright: Vec<String>,
    pub swift: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RuleScope {
    Repository,
}

fn default_true() -> bool {
    true
}

fn empty_options() -> serde_yaml::Value {
    serde_yaml::Value::Mapping(Default::default())
}
