use super::*;
use oxc_ast::ast::{TSLiteral, TSSignature, TSType};

/// Declarative annotation syntax; resolution uses the prepared import catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTypeSource {
    Reference(String),
    Scoped { scope: usize, source: Box<Self> },
    NonNullable(Box<Self>),
    Indexed(Box<Self>, String),
    Object(FxHashMap<String, Self>),
    Function(Vec<Option<Self>>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueTypeSources {
    pub parameters: FxHashMap<(CallableId, usize), ValueTypeSource>,
    pub declarations: FxHashMap<String, FxHashMap<String, ValueTypeSource>>,
    pub bindings: FxHashMap<usize, FxHashSet<String>>,
}

impl ImportCollector {
    pub(in super::super) fn record_complex_type_annotation(
        &mut self,
        owner: CallableId,
        index: usize,
        annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    ) {
        let Some(annotation) = annotation else {
            return;
        };
        if matches!(&annotation.type_annotation, TSType::TSTypeReference(reference) if reference.type_arguments.is_none())
        {
            return;
        }
        if let Some(source) = self.scoped_value_type_source(&annotation.type_annotation) {
            self.value_flow
                .type_sources
                .get_or_insert_with(Default::default)
                .parameters
                .insert((owner, index), source);
        }
    }

    pub(in super::super) fn scoped_value_type_source(
        &self,
        annotation: &TSType<'_>,
    ) -> Option<ValueTypeSource> {
        self.value_type_source(annotation)
            .map(|source| ValueTypeSource::Scoped {
                scope: self.current_lexical_scope_id(),
                source: Box::new(source),
            })
    }

    pub(in super::super) fn value_type_source(
        &self,
        annotation: &TSType<'_>,
    ) -> Option<ValueTypeSource> {
        match annotation {
            TSType::TSTypeReference(reference) => {
                let name = type_reference_name(reference)?;
                let root = name.split('.').next()?;
                if name == "NonNullable"
                    && !self.type_parameter_shadows(root)
                    && !self.type_binding_shadows(root)
                    && !self.local_binding_shadows(root)
                    && !self.imported_bindings.contains(root)
                    && !self.predeclared_imported_bindings.contains(root)
                {
                    let arguments = &reference.type_arguments.as_ref()?.params;
                    return (arguments.len() == 1)
                        .then(|| self.value_type_source(&arguments[0]))
                        .flatten()
                        .map(|source| ValueTypeSource::NonNullable(Box::new(source)));
                }
                // Module declarations are resolved in their declaring file. A
                // nested alias or generic cannot borrow an imported identity.
                if self.type_parameter_shadows(root)
                    || self
                        .type_local_stack
                        .iter()
                        .skip(1)
                        .any(|scope| scope.contains(root))
                    || reference.type_arguments.is_some()
                {
                    return None;
                }
                Some(ValueTypeSource::Reference(name))
            }
            TSType::TSIndexedAccessType(indexed) => {
                let TSType::TSLiteralType(key) = &indexed.index_type else {
                    return None;
                };
                let TSLiteral::StringLiteral(key) = &key.literal else {
                    return None;
                };
                Some(ValueTypeSource::Indexed(
                    Box::new(self.value_type_source(&indexed.object_type)?),
                    key.value.to_string(),
                ))
            }
            TSType::TSTypeLiteral(literal) => {
                let mut fields = fx_map();
                for signature in &literal.members {
                    let TSSignature::TSPropertySignature(property) = signature else {
                        continue;
                    };
                    if property.computed {
                        continue;
                    }
                    let Some(name) = write_targets::property_key(&property.key) else {
                        continue;
                    };
                    let Some(annotation) = &property.type_annotation else {
                        continue;
                    };
                    if let Some(source) = self.value_type_source(&annotation.type_annotation) {
                        fields.insert(name, source);
                    }
                }
                (!fields.is_empty()).then_some(ValueTypeSource::Object(fields))
            }
            TSType::TSFunctionType(function) if function.type_parameters.is_none() => {
                let mut sources: Option<Vec<Option<ValueTypeSource>>> = None;
                for (index, parameter) in function.params.items.iter().enumerate() {
                    let source = parameter
                        .type_annotation
                        .as_ref()
                        .and_then(|annotation| self.value_type_source(&annotation.type_annotation));
                    if source.is_some() && sources.is_none() {
                        sources = Some(vec![None; index]);
                    }
                    if let Some(sources) = &mut sources {
                        sources.push(source);
                    }
                }
                sources.map(ValueTypeSource::Function)
            }
            _ => None,
        }
    }

    pub(in super::super) fn record_type_source_declaration(
        &mut self,
        declaration: &TSTypeAliasDeclaration<'_>,
    ) {
        if !self.collect_value_flow || !self.function_stack.is_empty() {
            return;
        }
        if self.is_exported_top_level_type_name(declaration.id.name.as_str()) {
            self.value_flow.type_exports.push(ExportedBinding {
                specifier: None,
                local: declaration.id.name.to_string(),
                exported: declaration.id.name.to_string(),
            });
        }
        if declaration.type_parameters.is_some() {
            return;
        }
        if !matches!(declaration.type_annotation, TSType::TSTypeLiteral(_)) {
            return;
        }
        if let Some(ValueTypeSource::Object(fields)) =
            self.value_type_source(&declaration.type_annotation)
        {
            self.value_flow
                .type_sources
                .get_or_insert_with(Default::default)
                .declarations
                .insert(declaration.id.name.to_string(), fields);
        }
    }
}

impl ImportCollector {
    pub(in super::super) fn capture_value_type_scope(&mut self) {
        let scope = self.current_lexical_scope_id();
        if let (Some(sources), Some(names)) = (
            self.value_flow.type_sources.as_mut(),
            self.type_local_stack.last_mut(),
        ) {
            if !names.is_empty() {
                sources.bindings.insert(scope, std::mem::take(names));
            }
        }
    }
}
