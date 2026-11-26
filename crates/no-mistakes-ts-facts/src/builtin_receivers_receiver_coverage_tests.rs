use super::{callback_values, proven_facts};

#[test]
fn fallthrough_and_destructured_receivers_keep_their_proofs() {
    let source = "\
        const stable = ['a', 'b'];\n\
        Array.from(stable, (row) => row);\n\
        const { rows } = { rows: ['a'] };\n\
        rows.map((row) => row);\n\
        const date = new Date();\n\
        (true ? date : date).toISOString().replace('Z', '');\n\
        function take(when: Date = new Date()) {\n\
            return when.toISOString().replace('Z', '');\n\
        }\n";
    let facts = proven_facts(source);
    assert_eq!(
        callback_values(source, "Array.from(stable", &facts),
        vec![Some(vec![1])],
    );
    assert_eq!(callback_values(source, "rows.map", &facts), Vec::new());
    let conditional = callback_values(source, "(true ? date : date).toISOString().replace", &facts);
    assert!(
        !conditional.contains(&Some(vec![])),
        "a conditional date is not a string: {conditional:?}"
    );
    let dated = callback_values(source, "when.toISOString().replace", &facts);
    assert!(
        dated.contains(&Some(vec![])),
        "defaulted Date parameter should be a string leaf: {dated:?}"
    );
}
