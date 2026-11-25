use oxc_ast::{
    ast::{BindingPattern, UnaryOperator, VariableDeclarationKind},
    AstKind,
};
use oxc_semantic::{NodeId, Semantic};
use oxc_span::GetSpan;
use std::collections::BTreeSet;

mod property;

pub(super) fn reads(semantic: &Semantic<'_>, call: NodeId) -> Option<BTreeSet<String>> {
    let mut trace = Trace {
        semantic,
        pending: vec![(call, String::new())],
        paths: BTreeSet::new(),
    };
    while let Some((node, path)) = trace.pending.pop() {
        trace.follow(node, path)?;
    }
    Some(trace.paths)
}

struct Trace<'s, 'a> {
    semantic: &'s Semantic<'a>,
    pending: Vec<(NodeId, String)>,
    paths: BTreeSet<String>,
}

impl<'a> Trace<'_, 'a> {
    fn follow(&mut self, mut node: NodeId, mut path: String) -> Option<()> {
        let nodes = self.semantic.nodes();
        loop {
            let span = nodes.kind(node).span();
            let parent = nodes.parent_node(node);
            match parent.kind() {
                AstKind::StaticMemberExpression(member) if member.object.span() == span => {
                    property::push(&mut path, member.property.name.as_str())?;
                }
                AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
                    property::push(&mut path, &property::computed(&member.expression)?)?;
                }
                AstKind::ParenthesizedExpression(_)
                | AstKind::ChainExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSTypeAssertion(_) => {}
                AstKind::VariableDeclarator(declaration)
                    if declaration.init.as_ref().map(GetSpan::span) == Some(span) =>
                {
                    let variable = nodes.parent_id(parent.id());
                    if !matches!(
                        nodes.kind(variable),
                        AstKind::VariableDeclaration(variables)
                            if variables.kind == VariableDeclarationKind::Const
                    ) || matches!(
                        nodes.parent_kind(variable),
                        AstKind::ExportDeclaration(_) | AstKind::ExportNamedDeclaration(_)
                    ) {
                        return None;
                    }
                    return self.bind(&declaration.id, path);
                }
                AstKind::CallExpression(call) if call.callee.span() == span => return None,
                AstKind::NewExpression(call) if call.callee.span() == span => return None,
                AstKind::AssignmentExpression(assignment) if assignment.left.span() == span => {
                    return None;
                }
                AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::Delete => {
                    return None;
                }
                AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::AssignmentTargetPropertyProperty(_)
                | AstKind::AssignmentTargetRest(_)
                | AstKind::AssignmentTargetWithDefault(_)
                | AstKind::AwaitExpression(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::ExportSpecifier(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::PrivateFieldExpression(_)
                | AstKind::SpreadElement(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::UpdateExpression(_)
                | AstKind::YieldExpression(_) => return None,
                _ => return self.record(path),
            }
            node = parent.id();
        }
    }

    fn bind(&mut self, pattern: &BindingPattern<'a>, path: String) -> Option<()> {
        match pattern {
            BindingPattern::BindingIdentifier(binding) => {
                let symbol = binding.symbol_id.get()?;
                let scoping = self.semantic.scoping();
                let mut used = false;
                for reference in scoping.get_resolved_references(symbol) {
                    if reference.is_write() {
                        return None;
                    }
                    let flags = reference.flags();
                    if flags.is_value() && !flags.is_value_as_type() {
                        used = true;
                        self.pending.push((reference.node_id(), path.clone()));
                    }
                }
                if !used && !path.is_empty() {
                    self.record(path)?;
                }
            }
            BindingPattern::ObjectPattern(pattern) if pattern.rest.is_none() => {
                if pattern.properties.is_empty() {
                    return self.record(path);
                }
                for member in &pattern.properties {
                    let mut property_path = path.clone();
                    let key = property::destructured(&member.key)?;
                    property::push(&mut property_path, &key)?;
                    self.bind(&member.value, property_path)?;
                }
            }
            BindingPattern::AssignmentPattern(pattern) => {
                self.record(path.clone())?;
                self.bind(&pattern.left, path)?;
            }
            _ => return None,
        }
        Some(())
    }

    fn record(&mut self, path: String) -> Option<()> {
        if path.is_empty() {
            return None;
        }
        self.paths.insert(path);
        Some(())
    }
}
