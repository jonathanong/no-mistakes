use crate::fx::FxHashMap;
use crate::queue::extract_model::{JobIdQueue, LocalCheck};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::Visit;
use oxc_syntax::symbol::SymbolId;

mod expressions;
mod walk;

pub(crate) const KIND: &str = "queue-job-state-as-record";

#[derive(Debug, Clone)]
pub(crate) struct Site {
    pub origin: JobIdQueue,
    pub destination: Option<JobIdQueue>,
    pub check: LocalCheck,
}

#[derive(Clone)]
struct Origin {
    binding: JobIdQueue,
    at: u32,
}

#[derive(Clone)]
enum Value {
    Job(Origin),
    State(Origin, StateKind),
    Listing(Origin),
    MappedListing(Origin),
    Item(Origin),
    ItemData(Origin),
}

#[derive(Clone, Copy)]
enum StateKind {
    Name,
    Predicate,
}

#[derive(Default)]
struct Flow {
    values: FxHashMap<SymbolId, Value>,
    guards: Vec<Origin>,
    loops: Vec<u32>,
}

struct Collector<'queue, 'a> {
    queue: &'queue QueueVisitor<'a>,
    flow: Flow,
    sites: Vec<Site>,
}

pub(crate) fn needs_scoping(source: &str) -> bool {
    source.contains('\\')
        || (source.contains("getJob")
            || source.contains("getFailed")
            || source.contains("getCompleted"))
            && ["retry", "remove", "add"]
                .iter()
                .any(|method| source.contains(method))
}

pub(super) fn finish<'a>(visitor: &mut QueueVisitor<'a>, program: &'a Program<'a>) {
    if visitor.scoping.is_none() {
        return;
    }
    let mut collector = Collector {
        queue: visitor,
        flow: Flow::default(),
        sites: Vec::new(),
    };
    collector.visit_program(program);
    visitor.facts.job_state_sites = collector.sites;
}

impl Collector<'_, '_> {
    fn note_action(&mut self, call: &CallExpression<'_>) {
        let Some((receiver, method)) = expressions::method(call) else {
            return;
        };
        match method {
            "retry" | "remove" => self.note_job_action(call, receiver, method),
            "add" | "addBulk" => self.note_add(call, receiver, method),
            _ => {}
        }
    }

    fn note_job_action(
        &mut self,
        call: &CallExpression<'_>,
        receiver: &Expression<'_>,
        method: &str,
    ) {
        match self.value(receiver) {
            Some(Value::Job(origin))
                if self.flow.guards.iter().any(|guard| guard.at == origin.at) =>
            {
                self.sites.push(self.site(call, method, &origin, None));
            }
            Some(Value::Item(origin)) if method == "retry" => {
                self.sites.push(self.site(call, method, &origin, None));
            }
            _ => {}
        }
    }

    fn note_add(&mut self, call: &CallExpression<'_>, receiver: &Expression<'_>, method: &str) {
        let Some(destination) = self.queue_binding(receiver) else {
            return;
        };
        let data_origin = call
            .arguments
            .iter()
            .take(if method == "addBulk" { 1 } else { 2 })
            .filter_map(|argument| argument.as_expression())
            .find_map(|argument| match self.value(argument) {
                Some(Value::Item(origin) | Value::ItemData(origin)) => Some(origin),
                Some(Value::Listing(origin) | Value::MappedListing(origin))
                    if method == "addBulk" =>
                {
                    Some(origin)
                }
                _ => None,
            });
        let guards = self.flow.guards.iter().filter(|_| method == "add");
        let sites = data_origin
            .iter()
            .chain(guards)
            .map(|origin| self.site(call, method, origin, Some(destination.clone())))
            .collect::<Vec<_>>();
        self.sites.extend(sites);
    }

    fn site(
        &self,
        call: &CallExpression<'_>,
        method: &str,
        origin: &Origin,
        destination: Option<JobIdQueue>,
    ) -> Site {
        Site {
            origin: origin.binding.clone(),
            destination,
            check: LocalCheck {
                line: line_number(self.queue.source, call.span.start),
                kind: KIND.to_string(),
                job: None,
                message: format!(
                    "`{method}()` depends on retained queue job state. Decide what work is pending from a durable record, then enqueue it."
                ),
            },
        }
    }
}
