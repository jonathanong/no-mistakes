use super::*;
use oxc_ast_visit::Visit;
use std::{cell::RefCell, rc::Rc};
fn fixture(name: &str) -> String {
    std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers")
        .join(name)).unwrap()
}
fn probe(old: bool) -> Rc<RefCell<tests_support::Probe>> {
    Rc::new(RefCell::new(tests_support::Probe {
        old,
        ..Default::default()
    }))
}
struct Compare {
    expressions: usize,
    spilled: usize,
    rejected: usize,
}
impl<'a> Visit<'a> for Compare {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        let run = |old| {
            let p = probe(old);
            let mut effects = fresh_receiver_effects::FreshReceiverEffects::default();
            effects.rhs_probe = Some(p.clone());
            let result = inert_rhs(expression, &mut effects);
            (result, p, effects.rhs_visits)
        };
        let (old, a, visits_a) = run(true);
        let (new, b, visits_b) = run(false);
        assert_eq!(old, new);
        assert_eq!(a.borrow().trace, b.borrow().trace);
        assert_eq!(visits_a, visits_b);
        self.expressions += 1;
        self.spilled += usize::from(b.borrow().spill_growths > 0);
        self.rejected += usize::from(!new);
        oxc_ast_visit::walk::walk_expression(self, expression);
    }
}
#[test]
fn inert_pending_borrowed_stack_order() {
    let source = fixture("inert-rhs-pending-controls.mts");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    struct Pick;
    impl<'a> Visit<'a> for Pick {
        fn visit_expression(&mut self, expression: &Expression<'a>) {
            // References are used only during this callback; no AST lifetime cast.
            let mut pending = PendingExpressions::new(expression);
            assert!(std::ptr::eq(pending.pop().unwrap(), expression));
            assert!(pending.pop().is_none());
            assert!(!pending.push(expression));
            assert!(pending.push(expression));
            assert!(std::ptr::eq(pending.pop().unwrap(), expression));
            assert!(std::ptr::eq(pending.pop().unwrap(), expression));
            assert!(!pending.push(expression)); // spill-pop followed by push
            assert!(std::ptr::eq(pending.pop().unwrap(), expression));
            assert!(pending.pop().is_none());
        }
    }
    Pick.visit_program(&parsed.program);
    println!("STACK_CONTROL_COMPLETE");
}
#[test]
fn inert_pending_fullfacts_order_and_budget() {
    let mut phase = None;
    let mut differential = Compare {
        expressions: 0,
        spilled: 0,
        rejected: 0,
    };
    for name in [
        "inert-rhs-pending-controls.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "global-rhs-stress.mts",
        "global-phase-rhs-spread.mts",
        "mutable-let-pattern-rhs-rebind.mts",
        "mutable-let-pattern-lhs-rebind.mts",
    ] {
        let source = fixture(name);
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        differential.visit_program(&parsed.program);
        for demand in [false, true] {
            let a = probe(true);
            let b = probe(false);
            let old = crate::codebase::dependencies::extract::extract_import_facts_with_rhs_probe(
                &parsed.program,
                &source,
                false,
                false,
                demand,
                Some(a.clone()),
            );
            let new = crate::codebase::dependencies::extract::extract_import_facts_with_rhs_probe(
                &parsed.program,
                &source,
                false,
                false,
                demand,
                Some(b.clone()),
            );
            assert_eq!(old, new, "full ImportFacts {name} demand={demand}");
            assert_eq!(a.borrow().trace, b.borrow().trace);
            if !demand {
                assert_eq!(a.borrow().old_allocations + b.borrow().spill_growths, 0);
            }
            println!("FACTS_PARITY {name} demand={demand}");
            if name == "mutable-pattern-phase-scale.mts" && demand {
                phase = Some((a.borrow().old_allocations, b.borrow().spill_growths));
            }
        }
    }
    assert!(
        differential.expressions > 100 && differential.spilled > 0 && differential.rejected > 0
    );
    println!(
        "ORDERED_TRACE_COMPLETE expressions={} spilled={} rejected={}",
        differential.expressions, differential.spilled, differential.rejected
    );
    let (old, new) = phase.unwrap();
    println!("PHASE_PENDING_WORK old={old} new={new}");
    assert_eq!(old, 32, "saved phase must exercise all32 old allocations");
    println!("ALL_REQUIRED_CONTROLS_COMPLETE");
    let observed = if std::env::var_os("NM_INERT_EXPECT_RED").is_some() {
        old
    } else {
        new
    };
    assert_eq!(
        observed, 0,
        "INERT_PENDING_WORK_BUDGET: phase pending heap allocations must be zero"
    );
}

#[test]
fn inert_pending_request_and_rayon_isolation() {
    let run = |old| {
        let source = fixture("mutable-pattern-phase-scale.mts");
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let p = probe(old);
        let facts = crate::codebase::dependencies::extract::extract_import_facts_with_rhs_probe(
            &parsed.program,
            &source,
            false,
            false,
            true,
            Some(p.clone()),
        );
        assert_eq!(
            Rc::strong_count(&p),
            1,
            "request probe must not escape into facts"
        );
        let counts = (p.borrow().old_allocations, p.borrow().spill_growths);
        (facts, counts)
    };
    let ((old, a), (new, b)) = rayon::join(|| run(true), || run(false));
    assert_eq!(old, new);
    assert_eq!(a, (32, 0));
    assert_eq!(b, (0, 0));
    println!("REQUEST_RAYON_ISOLATION_COMPLETE");
}
