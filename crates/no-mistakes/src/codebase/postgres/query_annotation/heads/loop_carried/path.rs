//! Finite may-reach proof for one unconditional call/literal pair.
use super::condition_key::{key, Condition, Key};
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
mod correlation;

pub(super) enum Node {
    Scan(Span),
    Sequence(Vec<Self>),
    Branch(Option<Condition>, Box<Self>, Box<Self>),
    Backedge,
    Exit,
}

#[derive(Clone, Copy)]
enum Mode {
    LiteralToBackedge,
    BackedgeToRead,
}

#[derive(Clone, Copy)]
struct WalkInput<'a> {
    writes: &'a [Span],
    anchor: Span,
    mode: Mode,
}

#[derive(Default)]
struct Result {
    // Bit 0: dead value; bit 1: live value. Branches join by union.
    next: u8,
    hit: bool,
}

#[derive(Clone, Copy)]
struct Fixed<'a> {
    key: &'a Key,
    owners: &'a [usize],
    value: bool,
}

struct Fact<'a> {
    key: &'a Key,
    owners: Vec<usize>,
    value: bool,
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
                    _ => Self::Branch(key(&value.test), Box::new(consequent), Box::new(alternate)),
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

    fn walk<'a, F>(
        &'a self,
        incoming: u8,
        input: WalkInput<'_>,
        stable: &F,
        fixed: Option<Fixed<'_>>,
        known: &mut Vec<Fact<'a>>,
    ) -> Result
    where
        F: Fn(&Condition) -> Option<Vec<usize>>,
    {
        match self {
            Self::Scan(region) => {
                let mut events = input
                    .writes
                    .iter()
                    .copied()
                    .filter(|span| region.0 <= span.0 && span.1 <= region.1)
                    .filter(|span| *span != input.anchor)
                    .map(|span| (span, false))
                    .collect::<Vec<_>>();
                if region.0 <= input.anchor.0 && input.anchor.1 <= region.1 {
                    events.push((input.anchor, true));
                }
                events.sort_unstable_by_key(|(span, _)| *span);
                let mut state = incoming;
                let mut hit = false;
                for (_, is_anchor) in events {
                    if is_anchor {
                        match input.mode {
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
                    let step = node.walk(result.next, input, stable, fixed, known);
                    result.next = step.next;
                    result.hit |= step.hit;
                }
                result
            }
            Self::Branch(condition, left, right) => {
                let identity = condition.as_ref().and_then(stable);
                if let (Some(condition), Some(owners)) = (condition, identity.as_ref()) {
                    let truth = fixed
                        .filter(|fixed| fixed.key == &condition.key && fixed.owners == owners)
                        .map(|fixed| fixed.value)
                        .or_else(|| {
                            known
                                .iter()
                                .rev()
                                .find(|fact| fact.key == &condition.key && &fact.owners == owners)
                                .map(|fact| fact.value)
                        });
                    if let Some(truth) = truth {
                        let arm = if truth ^ condition.negative {
                            left
                        } else {
                            right
                        };
                        return arm.walk(incoming, input, stable, fixed, known);
                    }
                    known.push(Fact {
                        key: &condition.key,
                        owners: owners.clone(),
                        value: !condition.negative,
                    });
                    let left = left.walk(incoming, input, stable, fixed, known);
                    known.pop();
                    known.push(Fact {
                        key: &condition.key,
                        owners: owners.clone(),
                        value: condition.negative,
                    });
                    let right = right.walk(incoming, input, stable, fixed, known);
                    known.pop();
                    return Result {
                        next: left.next | right.next,
                        hit: left.hit || right.hit,
                    };
                }
                let left = left.walk(incoming, input, stable, fixed, known);
                let right = right.walk(incoming, input, stable, fixed, known);
                Result {
                    next: left.next | right.next,
                    hit: left.hit || right.hit,
                }
            }
            Self::Backedge => Result {
                next: 0,
                hit: matches!(input.mode, Mode::LiteralToBackedge) && incoming & 2 != 0,
            },
            Self::Exit => Result::default(),
        }
    }
}

fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}
