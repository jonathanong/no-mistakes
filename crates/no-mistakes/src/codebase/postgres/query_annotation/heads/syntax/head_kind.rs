//! Operations on syntax-only SQL head shapes.
use super::{Head, Span};

impl Head {
    pub fn is_constant(self) -> bool {
        matches!(
            self,
            Self::Literal | Self::Tag | Self::AppendTag | Self::AppendPrefixTag
        ) || matches!(self, Self::Text(value) if value.current != super::super::HeadProof::Unproven)
    }
    pub fn tagged(self) -> bool {
        matches!(self, Self::Tag | Self::AppendTag | Self::AppendPrefixTag)
            || matches!(self, Self::Text(value) if value.tagged)
    }
    pub fn appended(self) -> bool {
        matches!(self, Self::AppendTag | Self::AppendPrefixTag)
            || matches!(self, Self::Text(value) if value.appended)
    }
    pub fn proof(self, continued: bool) -> super::super::HeadProof {
        match self {
            Self::Literal | Self::Tag | Self::AppendTag => super::super::HeadProof::Literal,
            Self::AppendPrefixTag => super::super::HeadProof::Unproven,
            Self::Text(value) if value.opaque => super::super::HeadProof::Unproven,
            Self::Text(value) if continued => value.continued,
            Self::Text(value) => value.current,
            _ => super::super::HeadProof::Unproven,
        }
    }
    pub fn reference(self) -> Option<Span> {
        match self {
            Self::Reference(span)
            | Self::FieldReference(span)
            | Self::AppendReference(span)
            | Self::AppendPrefixReference(span) => Some(span),
            _ => None,
        }
    }
}
