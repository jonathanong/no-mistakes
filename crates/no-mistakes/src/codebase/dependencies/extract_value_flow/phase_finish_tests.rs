use super::*;

#[test]
fn finishing_restores_receipts_and_skips_writes_without_scope_receipts() {
    let catalog = Arc::new(PhaseCatalog {
        start: 10,
        end: 30,
        segments: vec![(10, 30, 1)],
    });
    let writes = vec![PhaseWrite {
        target: 12,
        name: "missing".into(),
        path: vec![Some("field".into())],
        phase: 1,
    }];
    let mut collector = ImportCollector::default();
    let identity = active_reference_phases::ActiveReferencePhases::new(
        (&catalog, 30),
        &writes,
        None,
        &collector.prior_member_mutations,
        None,
        false,
        Default::default(),
    );
    collector.fresh_receiver_effects.phases =
        Some(std::rc::Rc::new(std::cell::RefCell::new(identity)));
    collector.receiver_root_phases = Some(Box::new(root_phase_capture::RootPhaseCapture {
        catalog: Arc::clone(&catalog),
        first_changes: Default::default(),
        points: Default::default(),
        owner: None,
        conditional_depth: 0,
        events: vec![((0, "missing".into()), 1, ValueExpression::Unknown, true)],
    }));
    let phases = PreparedPhases {
        catalog,
        previous_deferred: None,
        defer_mutable: false,
        writes,
        previous_root_capture: None,
        root_capture: true,
        root_end: 30,
        root_guaranteed: true,
        owner: None,
        previous: Some(None),
        previous_identity: None,
    };
    let file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/ordinary-demand-boundaries.mts",
    );
    let source = std::fs::read_to_string(file).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let assignment = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            let Expression::AssignmentExpression(assignment) =
                crate::codebase::ts_source::unwrap_ts_wrappers(&statement.expression)
            else {
                return None;
            };
            matches!(
                assignment.left,
                AssignmentTarget::ObjectAssignmentTarget(_)
                    | AssignmentTarget::ArrayAssignmentTarget(_)
            )
            .then_some(assignment)
        })
        .unwrap();
    collector.collect_value_flow = true;
    assert!(collector.begin_phase_receipts(&assignment.left).is_some());
    let phases = Some(phases);
    collector.end_destructuring_lhs(&phases);
    assert!(
        !collector
            .destructuring_phase_receipts
            .as_ref()
            .unwrap()
            .active
    );
    // A receipt may disappear before a defensive finish; no lexical write may
    // be invented from a missing scope, and previous execution state still wins.
    collector.destructuring_phase_receipts = None;
    collector.end_destructuring_lhs(&phases);
    if let Some(phases) = phases {
        collector.finish_destructuring_phases(phases);
    }
    assert!(collector.fresh_receiver_effects.phases.is_none());
    assert_eq!(collector.destructuring_phase_owner, Some(None));
    assert!(collector.receiver_root_phases.is_none());
}
