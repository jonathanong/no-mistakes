use super::*;

#[test]
fn metadata_proof_boundaries_require_complete_same_file_row_summaries() {
    let Prepared {
        file,
        mut facts,
        graph,
        ..
    } = prepared("metadata-row-helpers.ts");
    let baseline = facts.get_ts_facts(&file).unwrap().clone();
    let call = baseline
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "getRowOrThrow")
        .unwrap();
    let value = ValueExpression::Call(call.offset, call.end);
    assert!(projection(
        &graph,
        &facts,
        &Bindings::new(&facts),
        &file,
        &value,
        &["table".into()]
    ));
    facts.ts.remove(&file);
    assert!(!projection(
        &graph,
        &facts,
        &Bindings::new(&facts),
        &file,
        &value,
        &["table".into()]
    ));
    for failure in ["parse", "operational", "incomplete"] {
        let mut ts = baseline.clone();
        match failure {
            "parse" => ts.parse_error = Some("missing row syntax".into()),
            "operational" => ts.operational_error = Some("source unavailable".into()),
            "incomplete" => ts.value_flow.incomplete_reason = Some("analysis budget".into()),
            _ => unreachable!(),
        }
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
        assert!(
            !projection(
                &graph,
                &facts,
                &Bindings::new(&facts),
                &file,
                &value,
                &["table".into()]
            ),
            "{failure}"
        );
    }
}
