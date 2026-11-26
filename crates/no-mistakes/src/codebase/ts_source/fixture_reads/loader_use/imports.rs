use oxc_ast::{
    ast::{FunctionType, ImportDeclaration, ImportDeclarationSpecifier, PropertyKey},
    AstKind,
};
use oxc_semantic::{NodeId, Semantic};
use std::collections::BTreeSet;

pub(super) struct LoaderNames<'a> {
    pub(super) roots: BTreeSet<&'a str>,
    pub(super) member_roots: BTreeSet<&'a str>,
    pub(super) member_leaves: BTreeSet<&'a str>,
    pub(super) modules: BTreeSet<&'a str>,
    pub(super) default_modules: BTreeSet<&'a str>,
}

impl<'a> LoaderNames<'a> {
    pub(super) fn from_callees(
        semantic: &'a Semantic<'a>,
        callees: &BTreeSet<&'a [String]>,
    ) -> Self {
        let roots: BTreeSet<&str> = callees.iter().map(|c| c[0].as_str()).collect();
        let member_roots: BTreeSet<&str> = callees
            .iter()
            .filter(|c| c.len() > 1)
            .map(|c| c[0].as_str())
            .collect();
        let member_leaves: BTreeSet<&str> = callees
            .iter()
            .filter(|c| c.len() > 1)
            .filter_map(|c| c.last().map(String::as_str))
            .collect();
        let mut modules = BTreeSet::new();
        let mut default_modules = BTreeSet::new();
        for node in semantic.nodes().iter() {
            let AstKind::ImportDeclaration(decl) = node.kind() else {
                continue;
            };
            if decl.import_kind.is_type() {
                continue;
            }
            let Some(source) = decl.source.value.as_str() else {
                continue;
            };
            for spec in decl.specifiers.iter().flatten() {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(named)
                        if !named.import_kind.is_type()
                            && roots.contains(named.local.name.as_str()) =>
                    {
                        modules.insert(source);
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default)
                        if roots.contains(default.local.name.as_str()) =>
                    {
                        modules.insert(source);
                        default_modules.insert(source);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns)
                        if roots.contains(ns.local.name.as_str()) =>
                    {
                        modules.insert(source);
                    }
                    _ => {}
                }
            }
        }
        Self {
            roots,
            member_roots,
            member_leaves,
            modules,
            default_modules,
        }
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.roots.contains(name) || self.member_leaves.contains(name)
    }

    pub(super) fn has_unsafe_import(&self, decl: &ImportDeclaration<'_>) -> bool {
        let is_default_module = decl
            .source
            .value
            .as_str()
            .is_some_and(|spec| self.default_modules.contains(spec));
        decl.specifiers.iter().flatten().any(|spec| match spec {
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                !self.member_roots.contains(ns.local.name.as_str())
            }
            ImportDeclarationSpecifier::ImportSpecifier(named) if !named.import_kind.is_type() => {
                let imported = named.imported.name().as_str();
                let local = named.local.name.as_str();
                (self.roots.contains(imported) && local != imported)
                    || self.member_leaves.contains(imported)
                    || (imported == "default" && is_default_module)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                is_default_module && !self.roots.contains(default.local.name.as_str())
            }
            _ => false,
        })
    }

    pub(super) fn is_allowed_binding(
        &self,
        semantic: &Semantic<'_>,
        node: NodeId,
        name: &str,
    ) -> bool {
        let nodes = semantic.nodes();
        let parent = nodes.parent_node(node);
        match parent.kind() {
            AstKind::ImportSpecifier(spec) => {
                !spec.import_kind.is_type() && spec.imported.name() == spec.local.name
            }
            AstKind::ImportDefaultSpecifier(_) => true,
            AstKind::ImportNamespaceSpecifier(_) => self.member_roots.contains(name),
            AstKind::Function(func) => {
                (func.declare || func.r#type == FunctionType::TSDeclareFunction)
                    && !is_exported(nodes.parent_kind(parent.id()))
            }
            AstKind::VariableDeclarator(_) => {
                let decl_id = nodes.parent_id(parent.id());
                matches!(nodes.kind(decl_id), AstKind::VariableDeclaration(decl) if decl.declare)
                    && !is_exported(nodes.parent_kind(decl_id))
            }
            _ => false,
        }
    }
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        PropertyKey::StringLiteral(lit) => lit.value.as_str(),
        _ => None,
    }
}

fn is_exported(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::ExportNamedDeclaration(_) | AstKind::ExportDefaultDeclaration(_)
    )
}
