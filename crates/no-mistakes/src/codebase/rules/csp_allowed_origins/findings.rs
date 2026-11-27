use super::{CspDirective, ParsedOrigin, RULE_ID};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) struct OriginRef {
    pub(crate) line: usize,
    pub(crate) directive: CspDirective,
    pub(crate) origin: ParsedOrigin,
}

#[derive(Clone, Copy)]
pub(crate) enum UnanalyzableReason {
    DynamicUrl(CspDirective),
    UnparsedUrl(CspDirective),
    DynamicLinkRel,
}

pub(crate) struct UnanalyzableRef {
    pub(crate) line: usize,
    pub(crate) reason: UnanalyzableReason,
}

#[derive(Default)]
pub(crate) struct CollectedOrigins {
    pub(crate) refs: Vec<OriginRef>,
    pub(crate) unanalyzable: Vec<UnanalyzableRef>,
    pub(crate) csp_strings: Vec<String>,
    pub(crate) csp_unanalyzable: Vec<usize>,
    /// Policies whose origin lists come from a named import. Expanded after every
    /// file's const-string exports have been recorded.
    pub(crate) deferred_policies: Vec<DeferredPolicy>,
    /// Sound `export const` string arrays in this module, keyed by export name.
    pub(crate) const_string_exports: BTreeMap<String, Vec<String>>,
}

pub(crate) struct DeferredPolicy {
    pub(crate) line: usize,
    pub(crate) parts: Vec<DeferredPart>,
}

pub(crate) enum DeferredPart {
    Text(String),
    Origins {
        module: PathBuf,
        export_name: String,
        separator: String,
    },
}

pub(super) fn origin_finding(
    file: &str,
    line: usize,
    directive: CspDirective,
    origin: &ParsedOrigin,
    csp_label: &str,
    custom_message: Option<&str>,
) -> RuleFinding {
    let origin_str = origin.display();
    let directive = directive.as_str();
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.to_owned(),
        line,
        message: custom_message.map(str::to_owned).unwrap_or_else(|| {
            format!(
                "Origin '{origin_str}' used for {directive} is not allowed by CSP in {csp_label}. Add '{origin_str}' to {directive} in {csp_label} or remove the third-party origin."
            )
        }),
        import: None,
        target: None,
    }
}

pub(super) fn unanalyzable_finding(
    file: &str,
    item: &UnanalyzableRef,
    csp_label: &str,
    custom_message: Option<&str>,
) -> RuleFinding {
    let message = custom_message.map(str::to_owned).unwrap_or_else(|| match item.reason {
        UnanalyzableReason::DynamicUrl(directive) => {
            let dir = directive.as_str();
            format!(
                "Dynamic URL used for {dir} cannot be statically verified against CSP in {csp_label}. Use a static origin literal (or a static origin prefix such as 'https://host/...') or set options.unanalyzable to 'advise' or 'ignore'."
            )
        }
        UnanalyzableReason::UnparsedUrl(directive) => {
            let dir = directive.as_str();
            format!(
                "Malformed network URL used for {dir} cannot be statically verified against CSP in {csp_label}. Use a valid origin URL (for example 'https://example.com/path') or set options.unanalyzable to 'advise' or 'ignore'."
            )
        }
        UnanalyzableReason::DynamicLinkRel => format!(
            "Dynamic <link> rel/as attribute cannot be mapped to a CSP directive for {csp_label}. Use static rel/as literals or set options.unanalyzable to 'advise' or 'ignore'."
        ),
    });
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.to_owned(),
        line: item.line,
        message,
        import: None,
        target: Some("unanalyzable".into()),
    }
}

pub(super) fn csp_unanalyzable_finding(
    file: &str,
    line: usize,
    custom_message: Option<&str>,
) -> RuleFinding {
    let message = custom_message.map(str::to_owned).unwrap_or_else(|| {
        format!(
            "Dynamic CSP policy expression in {file} cannot be statically resolved. Use static string literals (or a static array join with literal or nonce/hash template elements) or set options.unanalyzable to 'advise' or 'ignore'."
        )
    });
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.to_owned(),
        line,
        message,
        import: None,
        target: Some("unanalyzable".into()),
    }
}

pub(super) fn parse_error_finding(
    file: &str,
    csp_label: &str,
    custom_message: Option<&str>,
) -> RuleFinding {
    let message = custom_message.map(str::to_owned).unwrap_or_else(|| {
        format!(
            "Target file {file} failed to parse and cannot be statically verified against CSP in {csp_label}. Fix the syntax error or set options.unanalyzable to 'advise' or 'ignore'."
        )
    });
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.to_owned(),
        line: 1,
        message,
        import: None,
        target: Some("unanalyzable".into()),
    }
}
