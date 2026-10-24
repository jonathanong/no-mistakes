//! Finite may-reach proof for one unconditional call/literal pair.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(super) enum Node {
    Scan(Span),
    Sequence(Vec<Self>),
    Branch(Option<String>, Box<Self>, Box<Self>),
    Backedge,
    Exit,
}

#[derive(Clone, Copy)]
enum Mode {
    LiteralToBackedge,
    BackedgeToRead,
}

#[derive(Default)]
struct Result {
    // Bit 0: dead value; bit 1: live value. Branches join by union.
    next: u8,
    hit: bool,
}

impl Node {
    pub(super) fn from_statement(statement: &Statement<'_>) -> Self {
        match statement {
            Statement::BlockStatement(block) => {
                Self::Sequence(block.body.iter().map(Self::from_statement).collect())
            }
            Statement::ExpressionStatement(value) => Self::Scan(span(value.span)),
            Statement::VariableDeclaration(value) => Self::Scan(span(value.span)),
            Statement::IfStatement(value) => {
                let consequent = Self::from_statement(&value.consequent);
                let alternate = value
                    .alternate
                    .as_ref()
                    .map_or(Self::Sequence(Vec::new()), |alternate| {
                        Self::from_statement(alternate)
                    });
                let branch = match unwrap_ts_wrappers(&value.test) {
                    Expression::BooleanLiteral(test) if test.value => consequent,
                    Expression::BooleanLiteral(_) => alternate,
                    _ => Self::Branch(
                        condition_name(&value.test),
                        Box::new(consequent),
                        Box::new(alternate),
                    ),
                };
                Self::Sequence(vec![Self::Scan(span(value.test.span())), branch])
            }
            Statement::ContinueStatement(value) if value.label.is_none() => Self::Backedge,
            Statement::EmptyStatement(_) | Statement::DebuggerStatement(_) => {
                Self::Sequence(Vec::new())
            }
            // Break, return, throw, labelled continue, nested loops, and
            // unsupported syntax cannot carry this outer loop's backedge.
            _ => Self::Exit,
        }
    }

    pub(super) fn witness(&self, writes: &[Span], assignment: Span, read: Span) -> bool {
        let sensitive = writes
            .iter()
            .copied()
            .filter(|span| span.0 < read.0 || span.0 > assignment.0)
            .collect::<Vec<_>>();
        if self.correlated_change(&sensitive, &mut crate::fx::FxHashMap::default()) {
            return false;
        }
        let assigned = self.walk(1, writes, assignment, Mode::LiteralToBackedge);
        (assigned.hit || assigned.next & 2 != 0)
            && self.walk(2, writes, read, Mode::BackedgeToRead).hit
    }

    // Repeated tests of one identifier can be correlated. Joining states
    // independently could invent a path when both tests change the witness.
    fn correlated_change<'a>(
        &'a self,
        writes: &[Span],
        seen: &mut crate::fx::FxHashMap<&'a str, bool>,
    ) -> bool {
        match self {
            Self::Branch(name, left, right) => {
                if let Some(name) = name {
                    let changes = left.changes(writes) || right.changes(writes);
                    if let Some(previous) = seen.get_mut(name.as_str()) {
                        if *previous && changes {
                            return true;
                        }
                        *previous |= changes;
                    } else {
                        seen.insert(name, changes);
                    }
                }
                left.correlated_change(writes, seen) || right.correlated_change(writes, seen)
            }
            Self::Sequence(nodes) => nodes
                .iter()
                .any(|node| node.correlated_change(writes, seen)),
            _ => false,
        }
    }

    fn changes(&self, writes: &[Span]) -> bool {
        match self {
            Self::Scan(region) => writes
                .iter()
                .any(|span| region.0 <= span.0 && span.1 <= region.1),
            Self::Sequence(nodes) => nodes.iter().any(|node| node.changes(writes)),
            Self::Branch(_, left, right) => left.changes(writes) || right.changes(writes),
            Self::Backedge | Self::Exit => true,
        }
    }

    fn walk(&self, incoming: u8, writes: &[Span], anchor: Span, mode: Mode) -> Result {
        match self {
            Self::Scan(region) => {
                let mut events = writes
                    .iter()
                    .copied()
                    .filter(|span| region.0 <= span.0 && span.1 <= region.1)
                    .filter(|span| *span != anchor)
                    .map(|span| (span, false))
                    .collect::<Vec<_>>();
                if region.0 <= anchor.0 && anchor.1 <= region.1 {
                    events.push((anchor, true));
                }
                events.sort_unstable_by_key(|(span, _)| *span);
                let mut state = incoming;
                let mut hit = false;
                for (_, is_anchor) in events {
                    if is_anchor {
                        match mode {
                            Mode::LiteralToBackedge => state = 2,
                            Mode::BackedgeToRead => hit |= state & 2 != 0,
                        }
                    } else {
                        state = 1;
                    }
                }
                Result { next: state, hit }
            }
            Self::Sequence(nodes) => {
                let mut result = Result {
                    next: incoming,
                    hit: false,
                };
                for node in nodes {
                    if result.next == 0 {
                        break;
                    }
                    let step = node.walk(result.next, writes, anchor, mode);
                    result.next = step.next;
                    result.hit |= step.hit;
                }
                result
            }
            Self::Branch(_, left, right) => {
                let left = left.walk(incoming, writes, anchor, mode);
                let right = right.walk(incoming, writes, anchor, mode);
                Result {
                    next: left.next | right.next,
                    hit: left.hit || right.hit,
                }
            }
            Self::Backedge => Result {
                next: 0,
                hit: matches!(mode, Mode::LiteralToBackedge) && incoming & 2 != 0,
            },
            Self::Exit => Result::default(),
        }
    }
}

fn condition_name(value: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(value) => Some(value.name.to_string()),
        Expression::UnaryExpression(value)
            if value.operator == oxc_syntax::operator::UnaryOperator::LogicalNot =>
        {
            match unwrap_ts_wrappers(&value.argument) {
                Expression::Identifier(value) => Some(value.name.to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}
