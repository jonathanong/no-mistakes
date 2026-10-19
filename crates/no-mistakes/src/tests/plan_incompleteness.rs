use serde::{Deserialize, Serialize};
use std::fmt;

/// Why a plan's dependency coverage is unfinished.
///
/// The serde name is the public `incomplete[].kind` value. Package manifests,
/// JavaScript lockfiles, and .NET dependency inputs (`*.csproj`,
/// `Directory.Packages.props`, NuGet lockfiles) share one kind: the files
/// differ, and the condition is still a missing comparison baseline.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanIncompletenessKind {
    #[serde(rename = "dependency-baseline-missing")]
    DependencyBaselineMissing,
}

impl PlanIncompletenessKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DependencyBaselineMissing => "dependency-baseline-missing",
        }
    }
}

impl fmt::Display for PlanIncompletenessKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One reason a plan's dependency coverage is not complete.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlanIncompleteness {
    pub kind: PlanIncompletenessKind,
    pub files: Vec<String>,
}
