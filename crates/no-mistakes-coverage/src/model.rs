//! Canonical coverage report data structures.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Normalized coverage data across one or more source files.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub files: BTreeMap<String, FileCoverage>,
}

impl CoverageReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

/// Full-fidelity coverage metrics for a single source file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCoverage {
    pub functions: BTreeMap<String, FunctionCoverage>,
    pub branches: BTreeMap<BranchKey, u64>,
    pub lines: BTreeMap<u32, u64>,
}

impl FileCoverage {
    pub fn lines_found(&self) -> usize {
        self.lines.len()
    }

    pub fn lines_hit(&self) -> usize {
        self.lines.values().filter(|&&hits| hits > 0).count()
    }

    pub fn functions_found(&self) -> usize {
        self.functions.len()
    }

    pub fn functions_hit(&self) -> usize {
        self.functions.values().filter(|f| f.hits > 0).count()
    }

    pub fn branches_found(&self) -> usize {
        self.branches.len()
    }

    pub fn branches_hit(&self) -> usize {
        self.branches.values().filter(|&&hits| hits > 0).count()
    }
}

/// Function definition metadata and accumulated execution count.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionCoverage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    pub hits: u64,
}

/// Opaque branch identifier supporting both numeric and non-numeric block/branch ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchKey {
    pub line: u32,
    pub block: String,
    pub branch: String,
}

impl BranchKey {
    pub fn new(line: u32, block: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            line,
            block: block.into(),
            branch: branch.into(),
        }
    }

    pub fn as_lcov_key(&self) -> String {
        format!("{},{},{}", self.line, self.block, self.branch)
    }

    pub fn from_lcov_key(raw: &str) -> Option<Self> {
        let mut parts = raw.split(',');
        let line = parts.next()?.parse::<u32>().ok()?;
        let block = parts.next()?;
        let branch = parts.next()?;
        if block.is_empty() || branch.is_empty() || parts.next().is_some() {
            return None;
        }
        Some(Self::new(line, block, branch))
    }
}

fn cmp_natural_id(left: &str, right: &str) -> Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(a), Ok(b)) => a.cmp(&b).then_with(|| left.cmp(right)),
        _ => left.cmp(right),
    }
}

impl Ord for BranchKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| cmp_natural_id(&self.block, &other.block))
            .then_with(|| cmp_natural_id(&self.branch, &other.branch))
    }
}

impl PartialOrd for BranchKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for BranchKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_lcov_key())
    }
}

impl<'de> Deserialize<'de> for BranchKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::from_lcov_key(&raw).ok_or_else(|| serde::de::Error::custom("invalid branch key"))
    }
}

/// Classification of a non-fatal LCOV parse diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseDiagnosticKind {
    InvalidUtf8,
    InvalidPath,
    OrphanRecord,
    MalformedLine,
}

/// Scoped parse diagnostic recorded while tolerating malformed LCOV lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_path: Option<String>,
    pub kind: ParseDiagnosticKind,
    pub message: String,
}

/// Result of parsing an LCOV tracefile, preserving valid records alongside scoped diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedLcov {
    pub report: CoverageReport,
    pub diagnostics: Vec<ParseDiagnostic>,
}
