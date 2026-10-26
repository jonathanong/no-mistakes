//! Class-only metadata; ordinary import and callable projections stay unchanged.
use crate::codebase::dependencies::extract::{CallableId, InvocationKind, UnknownCall};
use crate::fx::FxHashSet;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
#[path = "ancestry_external.rs"]
mod external;

#[derive(Debug, Clone, Default)]
#[doc(hidden)]
pub struct AncestryFacts {
    pub classes: Vec<ClassShape>,
    pub ambient_modules: Vec<(String, Vec<CallableId>)>,
    pub declaration_aliases: Vec<(String, String)>,
    pub namespace_bodies: Vec<NamespaceBody>,
    pub export_assignment: Option<String>,
    /// `(module specifier, namespace identifier)` for `export =` inside `declare module`.
    pub ambient_module_assignments: Vec<(String, String)>,
    pub constructor_expressions: Vec<(u32, String)>,
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct ClassShape {
    pub id: CallableId,
    pub name: String,
    pub namespace: Vec<String>,
    pub bodies: Vec<u32>,
    pub has_base: bool,
    pub module_level: bool,
    pub module: Option<String>,
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct NamespaceBody {
    pub id: u32,
    pub namespace: Vec<String>,
    pub values: Vec<NamespaceValue>,
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct NamespaceValue {
    pub name: String,
    pub class: Option<CallableId>,
    pub exported: bool,
}

pub(super) fn collect(
    program: &Program<'_>,
    source: &str,
    unknown: &[UnknownCall],
) -> AncestryFacts {
    let mut visitor = Collector {
        source,
        ambient: program.source_type.is_typescript_definition(),
        unknown: unknown
            .iter()
            .filter(|call| call.invocation == InvocationKind::Construct)
            .map(|call| call.offset)
            .collect(),
        namespace: Vec::new(),
        external_module: None,
        global_modules: external::is_script(program),
        bodies: Vec::new(),
        function_depth: 0,
        block_depth: 0,
        class_depth: 0,
        facts: Default::default(),
    };
    visitor.visit_program(program);
    visitor.facts
}

struct Collector<'s> {
    source: &'s str,
    ambient: bool,
    unknown: FxHashSet<u32>,
    namespace: Vec<String>,
    external_module: Option<String>,
    global_modules: bool,
    bodies: Vec<u32>,
    function_depth: usize,
    block_depth: usize,
    class_depth: usize,
    facts: AncestryFacts,
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_ts_external_module_declaration(&mut self, module: &TSExternalModuleDeclaration<'a>) {
        external::visit(self, module);
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if self.ambient
            && self.namespace.is_empty()
            && self.external_module.is_none()
            && self.function_depth == 0
            && self.block_depth == 0
            && declaration.init.is_none()
        {
            self.facts
                .declaration_aliases
                .extend(super::ancestry_declarations::aliases(declaration));
        }
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.function_depth += 1;
        walk::walk_function(self, function, flags);
        self.function_depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, function: &ArrowFunctionExpression<'a>) {
        self.function_depth += 1;
        walk::walk_arrow_function_expression(self, function);
        self.function_depth -= 1;
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.block_depth += 1;
        walk::walk_block_statement(self, block);
        self.block_depth -= 1;
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(name) = &class.id {
            self.facts.classes.push(ClassShape {
                id: CallableId(class.span.start),
                name: name.name.to_string(),
                namespace: self.namespace.clone(),
                module: self.external_module.clone(),
                bodies: self.bodies.clone(),
                has_base: class.heritage.is_some(),
                module_level: self.function_depth == 0
                    && self.block_depth == 0
                    && self.class_depth == 0,
            });
        }
        self.class_depth += 1;
        walk::walk_class(self, class);
        self.class_depth -= 1;
    }

    fn visit_ts_namespace_declaration(&mut self, declaration: &TSNamespaceDeclaration<'a>) {
        let ambient = self.ambient;
        self.ambient |= declaration.declare;
        self.namespace.push(declaration.id.name.to_string());
        walk::walk_ts_namespace_declaration(self, declaration);
        self.namespace.pop();
        self.ambient = ambient;
    }

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        if self.namespace.is_empty() {
            walk::walk_ts_module_block(self, block);
            return;
        }
        self.facts
            .namespace_bodies
            .push(super::ancestry_namespaces::collect(
                block,
                &self.namespace,
                self.ambient,
            ));
        self.bodies.push(block.span.start);
        walk::walk_ts_module_block(self, block);
        self.bodies.pop();
    }

    fn visit_ts_export_assignment(&mut self, assignment: &TSExportAssignment<'a>) {
        external::record_assignment(self, assignment);
        walk::walk_ts_export_assignment(self, assignment);
    }

    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        let span = expression.callee.span();
        if let Some(text) = self
            .source
            .get(span.start as usize..span.end as usize)
            .filter(|_| self.unknown.contains(&expression.span.start))
        {
            // Bound diagnostics without changing the internal unknown-callee identity.
            let bounded: String = text.chars().take(160).collect();
            let display = bounded.split_whitespace().collect::<Vec<_>>().join(" ");
            self.facts
                .constructor_expressions
                .push((expression.span.start, display));
        }
        walk::walk_new_expression(self, expression);
    }
}
