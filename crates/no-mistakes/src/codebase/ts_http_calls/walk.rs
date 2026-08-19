use super::HttpCall;
use crate::codebase::ts_routes::refs::normalize_template;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression,
    Program,
};
use oxc_ast_visit::{walk, Visit};

const HTTP_VERBS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

pub(super) fn extract_http_calls_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    prefixes: &[&str],
) -> Vec<HttpCall> {
    let mut visitor = HttpCallVisitor {
        source,
        prefixes,
        http_ok: true,
        results: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.results
}

pub(crate) fn record_http_call(
    call: &CallExpression<'_>,
    source: &str,
    prefixes: &[&str],
    out: &mut Vec<HttpCall>,
) {
    let line = byte_offset_to_line(source, call.span.start as usize);
    let is_http_verb_call = match &call.callee {
        Expression::StaticMemberExpression(member) => {
            HTTP_VERBS.contains(&member.property.name.as_str())
        }
        _ => false,
    };
    let is_fetch_call = matches!(
        unwrap_ts_wrappers(&call.callee),
        Expression::Identifier(id) if id.name.as_str() == "fetch"
    );
    if !(is_http_verb_call || is_fetch_call) {
        return;
    }
    let Some(path) = static_path_arg(&call.arguments, 0) else {
        return;
    };
    if prefixes.iter().any(|prefix| path.starts_with(*prefix)) {
        out.push(HttpCall { path, line });
    }
}

pub(crate) fn export_default_allows_http(decl: &ExportDefaultDeclaration<'_>) -> bool {
    matches!(
        &decl.declaration,
        ExportDefaultDeclarationKind::FunctionDeclaration(_)
            | ExportDefaultDeclarationKind::ArrowFunctionExpression(_)
    )
}

struct HttpCallVisitor<'a, 'b> {
    source: &'a str,
    prefixes: &'b [&'a str],
    http_ok: bool,
    results: Vec<HttpCall>,
}

impl<'a> Visit<'a> for HttpCallVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.http_ok {
            record_http_call(call, self.source, self.prefixes, &mut self.results);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        let previous = self.http_ok;
        if !export_default_allows_http(decl) {
            self.http_ok = false;
        }
        walk::walk_export_default_declaration(self, decl);
        self.http_ok = previous;
    }
}

fn static_path_arg(args: &[Argument], index: usize) -> Option<String> {
    match args.get(index)? {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Argument::TemplateLiteral(tl) if tl.expressions.is_empty() => Some(normalize_template(tl)),
        _ => None,
    }
}
