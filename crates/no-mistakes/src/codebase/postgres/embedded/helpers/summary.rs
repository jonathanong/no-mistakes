//! Summaries of exported SQL helpers and the call sites that need them.

use super::split::{with_origin_entry, Marker};
use super::template::{Spliced, TemplateHelper};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::EmbeddedSqlCall;
use std::path::Path;

/// A string literal passed to a helper, plus where it was written.
///
/// `line == 0` means the location was not recorded (a value rebuilt inside a
/// helper, or a synthetic site). The splice then keeps the `${param}` line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LiteralArg {
    pub(crate) text: String,
    pub(crate) line: u32,
    pub(crate) origin: u32,
}

impl LiteralArg {
    pub(crate) fn bare(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            line: 0,
            origin: 0,
        }
    }

    fn splice<'a>(&'a self, caller: &Path) -> Spliced<'a> {
        Spliced {
            text: &self.text,
            at: (self.line > 0).then(|| Marker {
                line: self.line,
                path: Some(caller.to_path_buf()),
                origin: self.origin,
            }),
        }
    }
}

impl From<&str> for LiteralArg {
    fn from(text: &str) -> Self {
        Self::bare(text)
    }
}

impl From<String> for LiteralArg {
    fn from(text: String) -> Self {
        Self::bare(text)
    }
}

/// One argument of an imported-helper call, as far as the walk can tell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SiteArg {
    Literal(LiteralArg),
    Bool(bool),
    Null,
    Undefined,
    /// An object literal whose properties are all values the walk can classify.
    Object(Vec<(String, SiteArg)>),
    /// An array literal whose elements are all classifiable values.
    Array(Vec<SiteArg>),
    /// An identifier or number. It never contributes SQL text.
    Scalar,
    /// A SQL builder whose text precedes the call's hole.
    Builder,
}

/// An imported helper called from this file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedSite {
    /// The callee's local name; the projection looks up its import binding.
    pub(crate) local: String,
    pub(crate) args: Vec<SiteArg>,
    /// The call's result feeds an expression (not just a statement), so a
    /// builder helper must hand back the builder it was given.
    pub(crate) value_used: bool,
    /// Line of the call in the caller's file.
    pub(crate) line: u32,
}

/// Executor SQL that is complete once the holes of `sites` are filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingImportedSql {
    /// The executor call's source span, as recorded in `call_spans`.
    pub(crate) span: (u32, u32),
    /// The call with hole markers still in `sql_text`.
    pub(crate) template: EmbeddedSqlCall,
    pub(crate) sites: Vec<(usize, ImportedSite)>,
}

/// A helper function that is summarized once, where it is declared.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum HelperSummary {
    /// Fixed text; arguments never reach the SQL.
    Static(RecoveredSql),
    /// Untagged template over its own parameters.
    Template(TemplateHelper),
    /// Appends fixed fragments to one `SQLStatement` parameter and returns it.
    Builder(BuilderHelper),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct BuilderHelper {
    pub(crate) parameter_index: usize,
    pub(crate) parameter_count: usize,
    /// Everything the helper appends, in order.
    pub(crate) suffix: RecoveredSql,
}

impl HelperSummary {
    /// The text that fills a call's hole.
    ///
    /// Helper text is attributed to `helper`. A string literal spliced into a
    /// template is attributed to `caller` instead, at the literal's own line.
    pub(crate) fn fill(
        &self,
        site: &ImportedSite,
        helper: &Path,
        caller: &Path,
    ) -> Option<RecoveredSql> {
        let builders = site
            .args
            .iter()
            .filter(|arg| **arg == SiteArg::Builder)
            .count();
        let recovered = match self {
            Self::Static(sql) if builders == 0 => sql.clone(),
            Self::Template(template) if builders == 0 => {
                let literals = site
                    .args
                    .iter()
                    .map(|arg| match arg {
                        SiteArg::Literal(literal) => Some(literal.splice(caller)),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                template.apply(&literals)?
            }
            Self::Builder(builder)
                if builders == 1
                    && site.args.len() == builder.parameter_count
                    && site.args.get(builder.parameter_index) == Some(&SiteArg::Builder) =>
            {
                builder.suffix.clone()
            }
            _ => return None,
        };
        Some(with_origin_entry(recovered).with_source_file(helper))
    }
}
