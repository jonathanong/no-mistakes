use super::*;
#[path = "classes.rs"]
mod classes;
#[path = "exports.rs"]
mod exports;
#[path = "references.rs"]
mod references;
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, ImportedBindingKind, InvocationKind,
};
use crate::codebase::ts_source::facts::ancestry::ClassShape;

pub(super) fn prepare(session: &AnalysisSession, request: AncestryRequest<'_>) -> ClassAncestry {
    let mut builder = Builder {
        session,
        request,
        output: Default::default(),
        active: Default::default(),
        visited: Default::default(),
    };
    for path in builder.request.paths {
        let file = builder
            .request
            .facts
            .get_ts_facts(path)
            .expect("canonical graph preparation supplies every indexable file fact");
        for (offset, text) in &file.ancestry.constructor_expressions {
            builder
                .output
                .expressions
                .insert((path.clone(), *offset), text.clone());
        }
        if !builder.request.resolve_bases {
            continue;
        }
        for class in &file.ancestry.classes {
            if !class.has_base {
                builder.output.roots.insert(node(path, file, class));
            }
        }
        for call in &file.function_calls {
            let Some(id) = call
                .caller_id
                .filter(|_| call.is_callback && call.invocation == InvocationKind::Construct)
            else {
                continue;
            };
            // Canonical local/namespace binding resolution keeps precedence.
            if call.target_identity != CallTargetIdentity::ModuleExport {
                continue;
            }
            let Some(class) = file.ancestry.classes.iter().find(|class| class.id == id) else {
                continue;
            };
            let target = match super::namespaces::binding(file, class, &call.callee) {
                super::namespaces::Binding::Class(id) => file
                    .ancestry
                    .classes
                    .iter()
                    .find(|class| class.id == id)
                    .map(|class| builder.class(path, file, class)),
                super::namespaces::Binding::Opaque => None,
                super::namespaces::Binding::Absent => builder.reference(path, &call.callee),
            };
            if let Some(target) = target {
                builder.output.targets.insert((path.clone(), id), target);
            }
        }
    }
    builder
        .output
        .classes
        .sort_by(|a, b| (&a.file, a.callable_id).cmp(&(&b.file, b.callable_id)));
    builder.output
}

struct Builder<'a> {
    session: &'a AnalysisSession,
    request: AncestryRequest<'a>,
    output: ClassAncestry,
    active: FxHashSet<(PathBuf, String)>,
    visited: FxHashSet<(PathBuf, CallableId)>,
}

enum Facts<'a> {
    Borrowed(&'a TsFileFacts),
    Supplemental(Arc<TsFileFacts>),
}
impl std::ops::Deref for Facts<'_> {
    type Target = TsFileFacts;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(facts) => facts,
            Self::Supplemental(facts) => facts,
        }
    }
}

impl<'a> Builder<'a> {
    fn facts(&self, path: &Path) -> Facts<'a> {
        self.request
            .facts
            .get_ts_facts(path)
            .map(Facts::Borrowed)
            .unwrap_or_else(|| {
                Facts::Supplemental(self.session.declaration_facts(self.request.root, path))
            })
    }

    fn module(&self, importer: &Path, specifier: &str) -> Option<PathBuf> {
        if let Some(path) = self.request.resolver.resolve(specifier, importer) {
            // A project target never falls through to an unrelated installed package.
            return (self.request.facts.get_ts_facts(&path).is_some()
                || [".d.ts", ".d.mts", ".d.cts"]
                    .iter()
                    .any(|suffix| path.to_string_lossy().ends_with(suffix)))
            .then_some(path);
        }
        let sources = self
            .session
            .existing_sources_for(self.request.root)
            .unwrap_or_else(|| self.session.supplemental_sources.clone());
        // A real local candidate hidden by the graph's scope still owns this
        // specifier; do not reinterpret it as a same-named external package.
        if !specifier.starts_with('.')
            && self
                .request
                .resolver
                .resolution_candidates(specifier, importer)
                .iter()
                .any(|path| path.is_file())
        {
            return None;
        }
        package::resolve_in_context(specifier, importer, self.request.root, &sources)
    }
}

fn node(path: &Path, file: &TsFileFacts, class: &ClassShape) -> NodeId {
    NodeId::callable(path, class_scope(file, class), class.id)
}

fn class_scope<'a>(file: &'a TsFileFacts, class: &'a ClassShape) -> &'a str {
    file.callable_scope_ids
        .iter()
        .find(|(id, _)| *id == class.id)
        .map(|(_, scope)| scope.as_str())
        .unwrap_or(&class.name)
}

fn is_error_global(name: &str) -> bool {
    let name = ["globalThis.", "window.", "self.", "global."]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .unwrap_or(name);
    [
        "Error",
        "EvalError",
        "RangeError",
        "ReferenceError",
        "SyntaxError",
        "TypeError",
        "URIError",
        "AggregateError",
        "SuppressedError",
    ]
    .contains(&name)
}
