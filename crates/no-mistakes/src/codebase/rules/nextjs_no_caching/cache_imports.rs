use super::patterns::banned_next_cache_import;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};

pub(super) struct CacheImportEffects {
    pub(super) findings: Vec<(u32, String)>,
    pub(super) unstable_cache_bindings: Vec<String>,
    pub(super) namespaces: Vec<String>,
}

pub(super) fn effects(import: &ImportDeclaration<'_>) -> Option<CacheImportEffects> {
    if import.source.value.as_str() != "next/cache" {
        return None;
    }
    let Some(specifiers) = import.specifiers.as_ref() else {
        return Some(CacheImportEffects {
            findings: vec![(
                import.span.start,
                "next/cache side-effect imports are disabled; avoid Next.js cache APIs".to_string(),
            )],
            unstable_cache_bindings: Vec::new(),
            namespaces: Vec::new(),
        });
    };
    let mut effects = CacheImportEffects {
        findings: Vec::new(),
        unstable_cache_bindings: Vec::new(),
        namespaces: Vec::new(),
    };
    for specifier in specifiers {
        apply_specifier(specifier, &mut effects);
    }
    Some(effects)
}

fn apply_specifier(specifier: &ImportDeclarationSpecifier<'_>, effects: &mut CacheImportEffects) {
    match specifier {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
            effects
                .namespaces
                .push(spec.local.name.as_str().to_string());
            effects.findings.push((
                spec.span.start,
                "next/cache namespace imports are disabled; avoid Next.js cache APIs".to_string(),
            ));
        }
        ImportDeclarationSpecifier::ImportSpecifier(spec) => {
            let imported = spec.imported.name();
            if banned_next_cache_import(&imported) {
                if imported.as_str() == "unstable_cache" {
                    effects
                        .unstable_cache_bindings
                        .push(spec.local.name.as_str().to_string());
                }
                effects.findings.push((
                    spec.span.start,
                    format!("next/cache `{imported}` is disabled; avoid Next.js cache APIs"),
                ));
            }
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => effects.findings.push((
            spec.span.start,
            "next/cache default imports are disabled; avoid Next.js cache APIs".to_string(),
        )),
    }
}
