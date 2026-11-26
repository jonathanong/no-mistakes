use super::super::{ArgumentText, PathOrigin, PathProvenance, Projector};
use super::argument_source::ArgumentSource;
use super::call_args::MAX_ARGUMENTS;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(super) fn collect<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
    source: &ArgumentSource<'_>,
    out: &mut PathProvenance,
) -> (Vec<(usize, String, PathOrigin)>, Vec<ArgumentText>) {
    let mut members = Vec::new();
    let mut texts = Vec::new();
    for index in 0..MAX_ARGUMENTS {
        let Some(expression) = projector.argument_expression(call, index) else {
            break;
        };
        let literal_texts = projector
            .argument_object(expression)
            .and_then(|object| literal_texts(source, index, object));
        let recorded_literal = literal_texts.is_some();
        texts.extend(literal_texts.into_iter().flatten());
        MemberPaths {
            recorded_literal,
            offset: call.span.start,
            index,
            out,
            source,
            texts: &mut texts,
            members: &mut members,
        }
        .collect(projector, expression, "", 0);
    }
    members.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    texts.sort_by(|a, b| (&a.index, &a.property).cmp(&(&b.index, &b.property)));
    (members, texts)
}

struct MemberPaths<'o, 's> {
    recorded_literal: bool,
    source: &'o ArgumentSource<'s>,
    texts: &'o mut Vec<ArgumentText>,
    offset: u32,
    index: usize,
    out: &'o mut PathProvenance,
    members: &'o mut Vec<(usize, String, PathOrigin)>,
}

impl MemberPaths<'_, '_> {
    fn collect<'a>(
        &mut self,
        projector: &mut Projector<'a, '_>,
        expression: &Expression<'a>,
        prefix: &str,
        depth: usize,
    ) {
        if depth >= 8 {
            return;
        }
        let (paths, unknown) = projector.object_shape(expression, depth);
        if !unknown {
            if prefix.is_empty() {
                self.out
                    .arg_closed_objects
                    .entry(self.offset)
                    .or_default()
                    .push(self.index);
            } else {
                self.out
                    .arg_closed_members
                    .entry(self.offset)
                    .or_default()
                    .push((self.index, prefix.to_owned()));
            }
        }
        let mut paths: Vec<_> = paths.into_iter().collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, origin) in paths {
            let key_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            self.members.push((self.index, key_path.clone(), origin));
            if let Some(value) = projector.member_initializer(expression, &key, depth + 1) {
                if !prefix.is_empty() || !self.recorded_literal {
                    self.texts.push(super::call_args::argument_text(
                        self.source,
                        self.index,
                        Some(key_path.clone()),
                        value.span(),
                    ));
                }
                self.collect(projector, value, &key_path, depth + 1);
            }
        }
    }
}

fn literal_texts(
    source: &ArgumentSource<'_>,
    index: usize,
    object: &ObjectExpression<'_>,
) -> Option<Vec<ArgumentText>> {
    if object.properties.len() > 32 {
        return None;
    }
    let mut texts = crate::fx::FxHashMap::default();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.method || property.kind != PropertyKind::Init {
            return None;
        }
        if let Some(key) = crate::ast::property_key_name(&property.key) {
            let key = key.into_owned();
            texts.insert(
                key.clone(),
                super::call_args::argument_text(source, index, Some(key), property.value.span()),
            );
        }
    }
    Some(texts.into_values().collect())
}
