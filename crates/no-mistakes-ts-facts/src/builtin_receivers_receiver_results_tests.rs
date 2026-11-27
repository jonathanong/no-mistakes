use super::super::proven;
use super::{callback_values, proof_source, proven_facts};

#[test]
fn saved_receiver_results_follow_callbacks_or_string_leaves() {
    let source = proof_source("review-receiver-results.mts");
    let facts = proven_facts(&source);
    for call in [
        "SHARDS.find",
        "getResults().map",
        "resultRows.map",
        "rows.map",
        "failures.map",
        "main().finally",
    ] {
        assert_eq!(
            callback_values(&source, call, &facts),
            vec![Some(vec![0])],
            "{call}"
        );
    }
    for call in [
        "new Date().toISOString().replace",
        "date.toISOString().replace",
    ] {
        let found = callback_values(&source, call, &facts);
        assert!(
            found.contains(&Some(vec![])),
            "{call} should be a string leaf: {found:?}"
        );
    }
    assert!(
        proven(&source).iter().any(|call| call == "SHARDS.join"),
        "{source}"
    );
}

#[test]
fn unprovable_receivers_and_non_dates_stay_unresolved() {
    let source = "\
        new Map().toISOString().replace('a', 'b');\n\
        let rows = ['a']; rows.map((row) => row);\n\
        function *gen() { return ['a']; } gen().map((row) => row);\n\
        function mixed() { return ['a']; return 1; } mixed().map((row) => row);\n\
        function take(when: Date) { return when.toISOString().replace('Z', ''); }\n"
        .to_string();
    let facts = proven_facts(&source);
    for call in [
        "toISOString().replace('a'",
        "rows.map",
        "gen().map",
        "mixed().map",
    ] {
        assert_eq!(callback_values(&source, call, &facts), Vec::new(), "{call}");
    }
    let dated = callback_values(&source, "when.toISOString().replace", &facts);
    assert!(
        dated.contains(&Some(vec![])),
        "Date parameter toISOString().replace should be a string leaf: {dated:?}"
    );
}

#[test]
fn same_file_function_and_arrow_results_follow_callbacks() {
    let source = "\
        const listed = function (): string[] { return ['a']; };\n\
        (listed)().map((row) => row);\n\
        const tagged = (): string[] => ['a'];\n\
        tagged().map((row) => row);\n\
        const direct = () => ['a'];\n\
        direct().map((row) => row);\n\
        const blocked = () => { return ['a']; };\n\
        blocked().map((row) => row);\n\
        const shutdown = async () => undefined;\n\
        shutdown().finally(() => undefined);\n\
        const late = async function () { return 1; };\n\
        late().finally(() => undefined);\n\
        function items() {\n\
            function inner() { return 1; }\n\
            const hidden = () => 1;\n\
            return ['a'];\n\
        }\n\
        items().map((row) => row);\n\
        function a() { return ['a']; }\n\
        function b() { return a(); }\n\
        function c() { return b(); }\n\
        c().map((row) => row);\n\
        const notFn = ['a'];\n\
        notFn();\n\
        (function () { return ['a']; })();\n"
        .to_string();
    let facts = proven_facts(&source);
    for call in [
        "(listed)().map",
        "tagged().map",
        "direct().map",
        "blocked().map",
        "items().map",
        "c().map",
    ] {
        assert_eq!(
            callback_values(&source, call, &facts),
            vec![Some(vec![0])],
            "{call}"
        );
    }
    for call in ["shutdown().finally", "late().finally"] {
        assert_eq!(
            callback_values(&source, call, &facts),
            vec![Some(vec![0])],
            "{call}"
        );
    }
}

#[test]
fn uncalled_bodies_generators_and_date_casts_stay_conservative() {
    let unresolved = "\
        const gen = function* () { return ['a']; };\n\
        gen().map((row) => row);\n\
        function bare() { return; }\n\
        bare().map((row) => row);\n\
        function empty() {}\n\
        empty().map((row) => row);\n\
        async function* agen() { return ['a']; }\n\
        agen().map((row) => row);\n\
        function clash() { return ['a']; }\n\
        function clash() { return 1; }\n\
        clash().map((row) => row);\n\
        declare function ambient();\n\
        ambient().map((row) => row);\n"
        .to_string();
    let facts = proven_facts(&unresolved);
    for call in [
        "gen().map",
        "bare().map",
        "empty().map",
        "agen().map",
        "clash().map",
        "ambient().map",
    ] {
        assert_eq!(
            callback_values(&unresolved, call, &facts),
            Vec::new(),
            "{call}"
        );
    }
    let dates = "\
        const stamped = (0 as Date);\n\
        stamped.toISOString().replace('Z', '');\n\
        const asserted = (<Date>0);\n\
        asserted.toISOString().replace('Z', '');\n\
        (0 as Date).toISOString().replace('Z', '');\n\
        (<Date>0).toISOString().replace('Z', '');\n\
        const date = new Date();\n\
        date!.toISOString().replace('Z', '');\n\
        const bang = (new Date())!;\n\
        bang.toISOString().replace('Z', '');\n\
        function parenthesized(when: (Date)) {\n\
            return when.toISOString().replace('Z', '');\n\
        }\n"
    .to_string();
    let facts = proven_facts(&dates);
    for call in [
        "stamped.toISOString().replace",
        "asserted.toISOString().replace",
        "(0 as Date).toISOString().replace",
        "(<Date>0).toISOString().replace",
        "date!.toISOString().replace",
        "bang.toISOString().replace",
        "when.toISOString().replace",
    ] {
        let found = callback_values(&dates, call, &facts);
        assert!(
            found.contains(&Some(vec![])),
            "{call} should be a string leaf: {found:?}"
        );
    }
    let shadowed = "const Date = class {};\nconst local = new Date();\nlocal.toISOString().replace('Z', '');\n".to_string();
    let facts = proven_facts(&shadowed);
    assert_eq!(
        callback_values(&shadowed, "local.toISOString().replace", &facts),
        Vec::new()
    );
    let mixed = "\
        const when = new Date();\n\
        function take(when: string) { return when.toISOString().replace('Z', ''); }\n"
        .to_string();
    let facts = proven_facts(&mixed);
    assert_eq!(
        callback_values(&mixed, "when.toISOString().replace", &facts),
        Vec::new()
    );
}
