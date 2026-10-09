/// The innermost SQL clause containing a syntactic function call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SqlFunctionClause {
    Where,
    JoinOn,
    Having,
    SelectList,
    OrderBy,
    Values,
    Set,
    Default,
    Returning,
}

impl SqlFunctionClause {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "where" => Self::Where,
            "join-on" => Self::JoinOn,
            "having" => Self::Having,
            "select-list" => Self::SelectList,
            "order-by" => Self::OrderBy,
            "values" => Self::Values,
            "set" => Self::Set,
            "default" => Self::Default,
            "returning" => Self::Returning,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Where => "where",
            Self::JoinOn => "join-on",
            Self::Having => "having",
            Self::SelectList => "select-list",
            Self::OrderBy => "order-by",
            Self::Values => "values",
            Self::Set => "set",
            Self::Default => "default",
            Self::Returning => "returning",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Where => "WHERE",
            Self::JoinOn => "JOIN ON",
            Self::Having => "HAVING",
            Self::SelectList => "SELECT list",
            Self::OrderBy => "ORDER BY",
            Self::Values => "VALUES",
            Self::Set => "SET",
            Self::Default => "DEFAULT",
            Self::Returning => "RETURNING",
        }
    }
}
