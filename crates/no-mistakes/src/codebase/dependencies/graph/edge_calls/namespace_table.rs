use crate::codebase::dependencies::extract::{CallableId, NamespaceRoot};

/// The namespaces one file declares, indexed to resolve `new Errors.Inner.X()`
/// and a bare `new X()` written in the namespace body that declares `X`.
#[derive(Clone, Default)]
struct NamespaceTable {
    /// Member path (`Errors.Inner.X`) to the class's flat scope and parser id.
    members: FxHashMap<String, (String, CallableId)>,
    /// Every non-ambient namespace path, nested and dotted ones included.
    declared: FxHashSet<String>,
    roots: Vec<NamespaceRoot>,
    /// The innermost namespace around each construction written in a body.
    sites: FxHashMap<(Option<CallableId>, u32), String>,
}

enum NamespaceLookup<'a> {
    Member {
        scope: &'a str,
        id: CallableId,
    },
    /// `Errors.Missing`: `Errors` is declared here, but holds no such class.
    Missing {
        root: &'a str,
    },
    Absent,
}

/// `Errors.Inner` for `Errors.Inner.X`: the path without its last segment.
fn enclosing_path(path: &str) -> Option<&str> {
    path.rsplit_once('.').map(|(head, _)| head)
}

impl NamespaceTable {
    fn from_facts(file: &crate::codebase::ts_source::facts::TsFileFacts) -> Self {
        let facts = &file.namespaces;
        if facts.roots.is_empty() {
            return Self::default();
        }
        let member_ids: FxHashSet<CallableId> = facts.members.iter().map(|m| m.id).collect();
        let scopes: FxHashMap<CallableId, &String> = file
            .callable_scope_ids
            .iter()
            .filter(|(id, _)| member_ids.contains(id))
            .map(|(id, scope)| (*id, scope))
            .collect();
        Self {
            members: facts
                .members
                .iter()
                .filter_map(|member| {
                    let scope = scopes.get(&member.id)?;
                    Some((member.path.clone(), ((*scope).clone(), member.id)))
                })
                .collect(),
            declared: facts.declared.iter().cloned().collect(),
            roots: facts.roots.clone(),
            sites: facts
                .sites
                .iter()
                .map(|site| ((site.caller_id, site.offset), site.namespace.clone()))
                .collect(),
        }
    }

    fn is_empty(&self) -> bool {
        self.declared.is_empty()
    }

    /// The namespace around the construction at `offset` in `caller`, if any.
    fn context(&self, caller: Option<CallableId>, offset: u32) -> Option<&str> {
        self.sites.get(&(caller, offset)).map(String::as_str)
    }

    /// Resolves `callee` the way a namespace body does: the innermost
    /// enclosing namespace first, then each one outward, then the file's top
    /// level. `Errors.Missing` and `Errors.Missing.Factory` that find the
    /// declared `Errors` but no class are [`NamespaceLookup::Missing`].
    fn lookup(&self, context: Option<&str>, callee: &str) -> NamespaceLookup<'_> {
        let parents = std::iter::successors(enclosing_path(callee), |&path| enclosing_path(path));
        let mut missing = None;
        let mut context = context;
        loop {
            let qualify = |name: &str| match context {
                Some(context) => format!("{context}.{name}"),
                None => name.to_string(),
            };
            if let Some((scope, id)) = self.members.get(&qualify(callee)) {
                return NamespaceLookup::Member { scope, id: *id };
            }
            if missing.is_none() {
                let declared = parents
                    .clone()
                    .find_map(|parent| self.declared.get(&qualify(parent)));
                missing = declared.map(|declared| declared.split('.').next().unwrap_or(declared));
            }
            let Some(current) = context else { break };
            context = enclosing_path(current);
        }
        missing.map_or(NamespaceLookup::Absent, |root| NamespaceLookup::Missing {
            root,
        })
    }

    /// The root the file exports as `export`.
    fn root_exported_as(&self, export: &str) -> Option<&str> {
        self.roots
            .iter()
            .find(|root| root.exports.iter().any(|name| name == export))
            .map(|root| root.name.as_str())
    }
}
