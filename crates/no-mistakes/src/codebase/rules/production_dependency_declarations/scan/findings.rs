use super::*;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::rules::production_dependency_declarations::manifest::Classification;
use crate::codebase::rules::production_dependency_declarations::specifier;

pub(super) fn emit_finding(
    root: &Path,
    file: &Path,
    import: &FileImport,
    owning_package: &str,
    manifest: &PackageManifest,
    allowed_fields: &BTreeSet<String>,
    findings: &mut Vec<RuleFinding>,
) {
    if import.kind == ImportKind::Type {
        return; // erased by verbatimModuleSyntax; cannot fail at runtime
    }
    if specifier::is_relative(&import.specifier)
        || import.specifier.starts_with('#')
        || specifier::is_scheme_prefixed(&import.specifier)
    {
        return; // not a package.json dependency question
    }
    let Some(package_name) = specifier::package_name(&import.specifier) else {
        return;
    };
    if package_name == owning_package || specifier::is_node_builtin(&package_name) {
        return; // self-reference and Node builtins never need a declaration
    }
    match manifest.classify(&package_name, allowed_fields) {
        Classification::Allowed => {}
        Classification::DevOnly => findings.push(super::super::findings::dev_only(
            root,
            file,
            import.line,
            owning_package,
            &package_name,
            &import.specifier,
        )),
        Classification::Undeclared => findings.push(super::super::findings::undeclared(
            root,
            file,
            import.line,
            owning_package,
            &package_name,
            &import.specifier,
        )),
    }
}
