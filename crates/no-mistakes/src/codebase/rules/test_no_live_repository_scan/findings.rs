use super::super::RuleFinding;
use super::config::RuntimeAsset;
use super::RULE_ID;
use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use std::path::Path;

const FIX: &str = "enumerate a path under a fixtures directory or a mkdtemp directory the test owns, and run repository-wide policy in static CI";

pub(super) fn live(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    why: &str,
) -> RuleFinding {
    build(
        root,
        application,
        site,
        format!(
            "live repository scan: `{}` enumerates {why}, and a test root reaches it. Live trees change with unrelated files. Fix: {FIX}.",
            site.source_callee
        ),
        "live repository scan",
    )
}

pub(super) fn parameter(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    cause: &super::owned::Cause,
) -> RuleFinding {
    build(
        root,
        application,
        site,
        format!(
            "unresolved root: `{}` enumerates a directory whose root is a parameter, but {}. Fix: {}. As a last option, add `no-mistakes-disable-next-line {RULE_ID}` with a reason.",
            site.source_callee,
            cause.detail(root),
            cause.fix(),
        ),
        "unresolved root",
    )
}

pub(super) fn environment(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    cause: &super::owned::EnvCause,
) -> RuleFinding {
    build(
        root,
        application,
        site,
        cause.message(root, &site.source_callee),
        "unresolved root",
    )
}

pub(super) fn unresolved(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    why: &str,
) -> RuleFinding {
    build(
        root,
        application,
        site,
        format!(
            "unresolved root: `{}` enumerates a directory, but {why}, so it cannot be shown to be bounded. Fix: build a `const` fixture path or owned mkdtemp root inside the enumerating function, or supply it through every statically resolved test caller (use vi.stubEnv for env roots). As a last option, add `no-mistakes-disable-next-line {RULE_ID}` with a reason.",
            site.source_callee
        ),
        "unresolved root",
    )
}

pub(super) fn runtime_asset(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    asset: &RuntimeAsset,
) -> RuleFinding {
    build(
        root,
        application,
        site,
        format!(
            "runtime asset: `{}` enumerates `{}`, which `runtimeAssets` declares as shipped input ({}). Listed for review, not a repository scan. If this tree is policy or source, remove the declaration so the read is reported.",
            site.source_callee, asset.root, asset.reason
        ),
        "runtime asset",
    )
}

pub(super) fn unresolved_require(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    line: u32,
    specifier: &str,
    computed: bool,
) -> RuleFinding {
    let expression = if computed {
        format!("require({specifier})")
    } else {
        format!("require(\"{specifier}\")")
    };
    let why = if computed {
        "uses a computed module name that this analysis cannot resolve"
    } else {
        "binds a directory enumerator under a name this analysis cannot follow"
    };
    RuleFinding { causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, file),
        line: line as usize,
        message: application.message.clone().unwrap_or_else(|| format!(
            "unresolved root: `{expression}` {why}, so its roots cannot be shown to be bounded. Fix: use a statically named import and enumerate a path under a fixtures or owned temporary directory, or add `no-mistakes-disable-next-line {RULE_ID}` with a reason."
        )),
        import: Some(expression),
        target: Some("unresolved root".to_string()),
    }
}

fn build(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    default: String,
    target: &str,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &site.file),
        line: site.line as usize,
        message: application.message.clone().unwrap_or(default),
        import: Some(site.source_callee.clone()),
        target: Some(target.to_string()),
    }
}
