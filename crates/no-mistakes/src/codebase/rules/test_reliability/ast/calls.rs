use super::super::playwright_provenance::{ImportRef, WrapperCall};
use super::bindings::Runner::{Jest, Playwright, Vitest};
use super::playwright_bindings::ValueTrace;
use super::{literal_object, literal_string_is, property, retry_count, Collector, Origin, Shape};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::sync::Arc;

enum TestCall {
    VitestOptions,
    PlaywrightConfigure,
    PlaywrightSerial,
    JestRetry,
}

struct PendingPlaywright {
    import: Option<ImportRef>,
    binding: Option<Arc<str>>,
    wrappers: Vec<WrapperCall>,
    unresolved_reason: Option<&'static str>,
}

impl<'a> Visit<'a> for Collector<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((kind, origin, pending)) = self
            .test_call(call)
            .filter(|(_, origin, _)| self.demand.includes(*origin))
        {
            let start = self.locations.len();
            self.record_call(call, kind);
            if let Some(pending) = pending {
                for location in &mut self.locations[start..] {
                    location.origin = origin;
                    location.playwright_import.clone_from(&pending.import);
                    location.playwright_binding.clone_from(&pending.binding);
                    location.playwright_wrappers.clone_from(&pending.wrappers);
                    location.unresolved_reason = pending.unresolved_reason;
                }
            } else {
                for location in &mut self.locations[start..] {
                    location.origin = origin;
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

impl Collector<'_, '_> {
    fn test_call(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<(TestCall, Origin, Option<PendingPlaywright>)> {
        let Some((runner, members, vitest_global)) = self.bindings.call(&call.callee) else {
            return self.playwright_fixture_call(call);
        };
        let (kind, origin) = match (runner, members.as_slice()) {
            (Vitest, ["test" | "it" | "describe", ..]) => {
                let origin = if vitest_global {
                    Origin::VitestGlobal
                } else {
                    Origin::Standard
                };
                (TestCall::VitestOptions, origin)
            }
            (Playwright, ["test", "describe", "configure"]) => {
                (TestCall::PlaywrightConfigure, Origin::Playwright)
            }
            (Playwright, ["test", "describe", "serial", ..]) => {
                (TestCall::PlaywrightSerial, Origin::Playwright)
            }
            (Jest, ["jest", "retryTimes"]) => (TestCall::JestRetry, Origin::Standard),
            _ => return None,
        };
        Some((kind, origin, None))
    }

    fn playwright_fixture_call(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<(TestCall, Origin, Option<PendingPlaywright>)> {
        if !self.demand.playwright_tests {
            return None;
        }
        let member = unwrap_ts_wrappers(&call.callee).as_member_expression()?;
        let (kind, describe) = match member.static_property_name()?.as_str()? {
            "configure" => (TestCall::PlaywrightConfigure, member.object()),
            "serial" => (TestCall::PlaywrightSerial, member.object()),
            "only" | "skip" | "fixme" => {
                let serial = unwrap_ts_wrappers(member.object()).as_member_expression()?;
                if serial.static_property_name()?.as_str()? != "serial" {
                    return None;
                }
                (TestCall::PlaywrightSerial, serial.object())
            }
            _ => return None,
        };
        let describe = unwrap_ts_wrappers(describe).as_member_expression()?;
        if describe.static_property_name()?.as_str()? != "describe" {
            return None;
        }
        let receiver = describe.object();
        let pending = match self.bindings.playwright_value_origin(receiver) {
            ValueTrace::Resolved(init) => {
                (!init.import.is_playwright_test() || !init.wrappers.is_empty()).then(|| {
                    PendingPlaywright {
                        import: Some(init.import),
                        binding: receiver_label(receiver),
                        wrappers: init.wrappers,
                        unresolved_reason: None,
                    }
                })
            }
            ValueTrace::Unresolved(reason) => Some(PendingPlaywright {
                import: None,
                binding: receiver_label(receiver),
                wrappers: Vec::new(),
                unresolved_reason: Some(reason),
            }),
            ValueTrace::None => return None,
        };
        Some((kind, Origin::Playwright, pending))
    }

    fn record_call(&mut self, call: &CallExpression<'_>, kind: TestCall) {
        match kind {
            TestCall::VitestOptions => {
                for argument in call.arguments.iter().skip(1) {
                    if let Some(retry) = argument
                        .as_expression()
                        .and_then(literal_object)
                        .and_then(|object| property(object, "retry"))
                    {
                        self.property_retry("Vitest retry", retry, None);
                    }
                }
            }
            TestCall::PlaywrightConfigure => {
                if let Some(options) = call
                    .arguments
                    .first()
                    .and_then(|arg| arg.as_expression())
                    .and_then(literal_object)
                {
                    if let Some(retries) = property(options, "retries") {
                        self.property_retry("Playwright retries", retries, None);
                    }
                    if let Some(mode) = property(options, "mode")
                        .filter(|mode| literal_string_is(&mode.value, "serial"))
                    {
                        self.record(
                            Shape::Serial("Playwright describe serial mode"),
                            mode.key.span(),
                            None,
                        );
                    }
                }
            }
            TestCall::PlaywrightSerial => self.record(
                Shape::Serial("Playwright test.describe.serial"),
                call.callee.span(),
                None,
            ),
            TestCall::JestRetry => {
                let retries = call
                    .arguments
                    .first()
                    .and_then(|arg| arg.as_expression())
                    .and_then(retry_count);
                self.record(
                    Shape::Retry("Jest retryTimes", retries),
                    call.callee.span(),
                    None,
                );
            }
        }
    }
}

fn receiver_label(expr: &Expression<'_>) -> Option<Arc<str>> {
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(id) => Some(Arc::from(id.name.as_str())),
        other => {
            let member = other.as_member_expression()?;
            let prop = member.static_property_name()?.as_str()?;
            let Expression::Identifier(obj) = unwrap_ts_wrappers(member.object()) else {
                return None;
            };
            Some(Arc::from(format!("{}.{prop}", obj.name.as_str())))
        }
    }
}
