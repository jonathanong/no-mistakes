//! Binding-only SQL tags; sql-template-strings never splices interpolation text.
use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use oxc_ast::ast::{SimpleAssignmentTarget, TaggedTemplateExpression};

#[derive(Default)]
pub(super) struct TemplateBinds {
    references: FxHashMap<Span, (usize, bool)>,
    invalid: FxHashSet<usize>,
    lines: Vec<u32>,
}
impl TemplateBinds {
    pub fn new(module: &TypeScriptModuleFacts, source: &str) -> Self {
        let tags = module
            .imports
            .iter()
            .filter(|import| import.specifier == "sql-template-strings" && !import.type_only)
            .flat_map(|import| &import.bindings)
            .filter(|binding| {
                !binding.type_only
                    && (binding.kind == "default"
                        || binding.kind == "namespace"
                        || (binding.kind == "named"
                            && matches!(binding.imported.as_str(), "default" | "SQL")))
            })
            .map(|binding| (binding.binding_id, binding.kind == "namespace"))
            .collect::<FxHashMap<_, _>>();
        Self {
            references: module
                .bindings
                .iter()
                .filter_map(|binding| tags.get(&binding.id).map(|namespace| (binding, *namespace)))
                .flat_map(|(binding, namespace)| {
                    binding
                        .references
                        .iter()
                        .filter(|usage| usage.runtime)
                        .map(move |usage| {
                            ((usage.span.start, usage.span.end), (binding.id, namespace))
                        })
                })
                .collect(),
            invalid: FxHashSet::default(),
            lines: source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index as u32))
                .collect(),
        }
    }
    pub fn write_span(&mut self, at: Span) {
        if let Some((id, _)) = self.references.get(&at) {
            self.invalid.insert(*id);
        }
    }
    pub fn write(&mut self, target: &SimpleAssignmentTarget<'_>) {
        let at = match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => Some(span(id.span)),
            SimpleAssignmentTarget::StaticMemberExpression(value) => {
                chain(&value.object).map(|v| v.0)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(value) => {
                chain(&value.object).map(|v| v.0)
            }
            SimpleAssignmentTarget::TSAsExpression(value) => chain(&value.expression).map(|v| v.0),
            SimpleAssignmentTarget::TSSatisfiesExpression(value) => {
                chain(&value.expression).map(|v| v.0)
            }
            SimpleAssignmentTarget::TSNonNullExpression(value) => {
                chain(&value.expression).map(|v| v.0)
            }
            SimpleAssignmentTarget::TSTypeAssertion(value) => chain(&value.expression).map(|v| v.0),
            _ => None,
        };
        if let Some(at) = at {
            self.write_span(at);
        }
    }
    pub fn validate(&self, input: &mut Input) {
        if matches!(input, Input::Sql { parameterized: Some(id), .. } if self.invalid.contains(id))
        {
            *input = Input::Unknown;
        }
    }
    pub fn input(&self, tagged: &TaggedTemplateExpression<'_>) -> Input {
        let Some((root, parts)) = chain(&tagged.tag) else {
            return Input::Unknown;
        };
        let Some(&(id, namespace)) = self.references.get(&root) else {
            return Input::Unknown;
        };
        if self.invalid.contains(&id)
            || !(if namespace {
                parts.len() == 2 && matches!(parts[1].as_str(), "default" | "SQL")
            } else {
                parts.len() == 1
            })
        {
            return Input::Unknown;
        }
        let mut text = String::new();
        let mut positions = Vec::new();
        let (mut sql_line, mut sql_column) = (1, 1);
        for (index, quasi) in tagged.quasi.quasis.iter().enumerate() {
            let Some(cooked) = &quasi.value.cooked else {
                return Input::Unknown;
            };
            if !super::literal_projection::literal_matches(quasi.value.raw.as_str(), cooked) {
                return Input::Unknown;
            }
            positions.push(super::super::EmbeddedSqlSourcePosition {
                sql_line,
                sql_column,
                source_line: self.lines.partition_point(|line| *line < quasi.span.start) as u32 + 1,
                source_path: None,
            });
            for character in cooked.chars() {
                if character == '\n' {
                    sql_line += 1;
                    sql_column = 1;
                } else {
                    sql_column += 1;
                }
            }
            text.push_str(cooked);
            if index < tagged.quasi.expressions.len() {
                let placeholder = format!("${}", index + 1);
                sql_column += placeholder.len() as u32;
                text.push_str(&placeholder);
            }
        }
        Input::Sql {
            text,
            source_span: span(tagged.quasi.span),
            holes: Vec::new(),
            parameterized: Some(id),
            positions,
        }
    }
}

pub(super) fn safe_constants(module: &TypeScriptModuleFacts, calls: &[Call]) -> FxHashSet<Span> {
    let direct = calls
        .iter()
        .filter_map(|call| match call.input {
            Input::Reference(at) => Some(at),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
    module
        .bindings
        .iter()
        .filter(|binding| {
            binding
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .all(|usage| direct.contains(&(usage.span.start, usage.span.end)))
        })
        .map(|binding| (binding.span.start, binding.span.end))
        .collect()
}
