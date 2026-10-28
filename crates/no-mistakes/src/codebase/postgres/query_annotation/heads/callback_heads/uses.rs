use super::Payload;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, Expression};
use oxc_ast_visit::Visit;
mod visit;

pub(super) struct Uses<'a, 't> {
    params: Vec<&'a str>,
    callback: usize,
    allowed_callback: FxHashSet<u32>,
    allowed_payload: FxHashSet<u32>,
    discarded_appends: FxHashSet<u32>,
    references: Vec<(&'a str, u32)>,
    payload: Option<Payload>,
    count: usize,
    valid: bool,
    nodes: usize,
    looped: bool,
    trusted_tags: &'t FxHashSet<super::Span>,
}

impl<'a, 't> Uses<'a, 't> {
    pub(super) fn inspect(
        body: &ast::FunctionBody<'a>,
        params: Vec<&'a str>,
        callback: usize,
        trusted_tags: &'t FxHashSet<super::Span>,
    ) -> Option<Payload> {
        let mut uses = Self {
            params,
            callback,
            allowed_callback: FxHashSet::default(),
            allowed_payload: FxHashSet::default(),
            discarded_appends: FxHashSet::default(),
            references: Vec::new(),
            payload: None,
            count: 0,
            valid: true,
            nodes: 0,
            looped: false,
            trusted_tags,
        };
        uses.visit_function_body(body);
        let payload = uses.payload?;
        let payload_name = match payload {
            Payload::Parameter(index) => Some(uses.params[index]),
            Payload::Fixed | Payload::FixedBuilder => None,
        };
        if !uses.valid
            || uses.count == 0
            || uses.count > 16
            || uses.looped && matches!(payload, Payload::Parameter(_))
        {
            return None;
        }
        for (name, span) in uses.references {
            if name == uses.params[callback] && !uses.allowed_callback.contains(&span)
                || payload_name == Some(name) && !uses.allowed_payload.contains(&span)
            {
                return None;
            }
        }
        Some(payload)
    }

    fn payload(&self, arg: &Expression<'a>) -> Option<Payload> {
        match unwrap_ts_wrappers(arg) {
            Expression::Identifier(id) => self
                .params
                .iter()
                .position(|name| *name == id.name && *name != self.params[self.callback])
                .map(Payload::Parameter),
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
                if super::super::syntax::head(arg).proof(false)
                    == super::super::HeadProof::Literal =>
            {
                Some(Payload::Fixed)
            }
            _ if super::super::trusted_append::tag(arg)
                .is_some_and(|tag| self.trusted_tags.contains(&tag))
                && matches!(
                    super::super::syntax::head(arg),
                    super::super::syntax::Head::Tag
                        | super::super::syntax::Head::AppendTag
                        | super::super::syntax::Head::AppendPrefixTag
                ) =>
            {
                Some(Payload::FixedBuilder)
            }
            _ => None,
        }
    }
}
