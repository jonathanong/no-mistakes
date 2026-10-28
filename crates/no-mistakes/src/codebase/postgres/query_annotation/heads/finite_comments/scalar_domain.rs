//! One exact private-helper call census per scalar-name owner, including failures.
use super::scalar_ownership::{only, parameter, stable};
use super::{Argument, CallSite, Function, Root, Span};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct Index<'a> {
    pub declarations: FxHashMap<Span, Option<&'a ModuleBinding>>,
    functions: FxHashMap<Span, Option<&'a Function>>,
    calls: FxHashMap<usize, Vec<&'a CallSite>>,
    roots: FxHashSet<usize>,
    exports: FxHashSet<&'a str>,
    memo: FxHashMap<usize, Option<Vec<String>>>,
    constructors: FxHashMap<usize, bool>,
    native: Vec<&'a ModuleBinding>,
    native_complete: bool,
}
impl<'a> Index<'a> {
    pub fn new(
        syntax: &'a Syntax,
        bindings: &Bindings<'_>,
        module: &'a TypeScriptModuleFacts,
    ) -> Self {
        let mut declarations = FxHashMap::default();
        for binding in &module.bindings {
            declarations
                .entry((binding.span.start, binding.span.end))
                .and_modify(|owner| *owner = None)
                .or_insert(Some(binding));
        }
        let mut functions = FxHashMap::default();
        for function in &syntax.finite_functions {
            functions
                .entry(function.body)
                .and_modify(|owner| *owner = None)
                .or_insert(Some(function));
        }
        let mut calls: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for call in &syntax.finite_calls {
            if let Some(owner) = bindings.owner(call.callee) {
                calls.entry(owner.id).or_default().push(call);
            }
        }
        let native_ids = module
            .imports
            .iter()
            .filter(|import| import.specifier == "sql-template-strings" && !import.type_only)
            .flat_map(|import| &import.bindings)
            .filter(|binding| !binding.type_only)
            .map(|binding| binding.binding_id)
            .collect::<FxHashSet<_>>();
        let native = module
            .bindings
            .iter()
            .filter(|binding| binding.runtime && native_ids.contains(&binding.id))
            .collect::<Vec<_>>();
        let native_complete = native.len() == native_ids.len();
        Self {
            declarations,
            native,
            native_complete,
            functions,
            calls,
            roots: module
                .scopes
                .iter()
                .filter(|scope| scope.parent_id.is_none())
                .map(|scope| scope.id)
                .collect(),
            exports: module
                .exports
                .iter()
                .filter(|export| !export.type_only && export.specifier.is_none())
                .map(|export| export.local.as_str())
                .collect(),
            memo: FxHashMap::default(),
            constructors: FxHashMap::default(),
        }
    }
    pub fn exported(&self, binding: &ModuleBinding) -> bool {
        self.roots.contains(&binding.scope_id) && self.exports.contains(binding.name.as_str())
    }
    pub fn values(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        root: &Root,
    ) -> Option<&[String]> {
        let function = self.functions.get(&root.function).copied().flatten()?;
        parameter(bindings, root.reference, function.parameter)?;
        let helper = self
            .declarations
            .get(&function.declaration)
            .copied()
            .flatten()?;
        if !self.memo.contains_key(&helper.id) {
            let values = self.domain(syntax, bindings, root, function, helper);
            self.memo.insert(helper.id, values);
        }
        self.memo.get(&helper.id)?.as_deref()
    }
    fn domain(
        &self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        root: &Root,
        function: &Function,
        helper: &ModuleBinding,
    ) -> Option<Vec<String>> {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.scalar_name_helper_censuses", 1);
        }
        let parameter = self
            .declarations
            .get(&function.parameter)
            .copied()
            .flatten()?;
        if !function.scalar_plain
            || helper.imported
            || !helper.runtime
            || self.exported(helper)
            || !bindings.declaration_safe(root.function)
            || !stable(syntax, bindings, helper)
            || !stable(syntax, bindings, parameter)
            || !only(parameter, &[root.reference])
            || bindings
                .owner(root.reference)
                .is_none_or(|owner| owner.id != parameter.id)
            || syntax.contexts.get(&root.reference).copied().flatten() != Some(root.function)
        {
            return None;
        }
        let calls = self.calls.get(&helper.id)?;
        if calls.is_empty()
            || calls.len() > 16
            || !only(
                helper,
                &calls.iter().map(|call| call.callee).collect::<Vec<_>>(),
            )
        {
            return None;
        }
        let mut values = Vec::new();
        for call in calls {
            if !call.valid
                || call.count != function.count
                || call.context.is_some_and(|context| {
                    root.function.0 <= context.0 && context.1 <= root.function.1
                })
            {
                return None;
            }
            let Argument::Literal(value) = &call.first else {
                return None;
            };
            if value.trim().is_empty() || value.contains("/*") || value.contains("*/") {
                return None;
            }
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
        Some(values)
    }
    pub fn constructor(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        reference: Span,
    ) -> Option<usize> {
        let owner = bindings.owner(reference)?;
        if !self.native_complete || !bindings.trusted_import(owner.id) {
            return None;
        }
        for &native in &self.native {
            if self.constructors.contains_key(&native.id) {
                continue;
            }
            let safe = super::scalar_ownership::native_safe(
                syntax,
                bindings,
                native,
                self.exported(native),
            );
            self.constructors.insert(native.id, safe);
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.scalar_name_constructor_censuses", 1);
            }
        }
        if self.constructors.values().any(|safe| !safe) {
            return None;
        }
        self.constructors
            .get(&owner.id)
            .copied()
            .unwrap_or(false)
            .then_some(owner.id)
    }
}
