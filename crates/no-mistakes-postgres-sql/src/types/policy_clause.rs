#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub enum SqlPolicyClause {
    CreateExtensionVersion,
    DropCascade,
}

impl SqlPolicyClause {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "create-extension-version" => Some(Self::CreateExtensionVersion),
            "drop-cascade" => Some(Self::DropCascade),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::CreateExtensionVersion => "create-extension-version",
            Self::DropCascade => "drop-cascade",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub struct SqlPolicyClauseUse {
    pub clause: SqlPolicyClause,
    pub line: usize,
    pub column: usize,
}
