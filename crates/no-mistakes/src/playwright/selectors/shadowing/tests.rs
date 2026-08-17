use super::*;
use oxc_span::Span;

#[test]
fn identifier_reassignment_uses_identifier_boundaries_and_assignment_operator() {
    assert!(has_identifier_reassignment("dataPw = makeId();", "dataPw"));
    assert!(has_identifier_reassignment(
        "data$Pw = makeId();",
        "data$Pw"
    ));
    assert!(!has_identifier_reassignment("xdataPw = 1;", "dataPw"));
    assert!(!has_identifier_reassignment("dataPwx = 1;", "dataPw"));
    assert!(has_identifier_reassignment("dataPw = 1;", "dataPw"));
    assert!(has_identifier_reassignment("dataPw += 1;", "dataPw"));
    assert!(has_identifier_reassignment("dataPw++", "dataPw"));
    assert!(has_identifier_reassignment("++dataPw", "dataPw"));
    assert!(has_identifier_reassignment("dataPw += '-x';", "dataPw"));
    assert!(has_identifier_reassignment(
        "dataPw ??= makeId();",
        "dataPw"
    ));
    assert!(has_identifier_reassignment("dataPw++;", "dataPw"));
    assert!(has_identifier_reassignment("--dataPw;", "dataPw"));
    assert!(!has_identifier_reassignment("dataPw === 'save';", "dataPw"));
    assert!(!has_identifier_reassignment("dataPw == 'save';", "dataPw"));
    assert!(!has_identifier_reassignment(
        "// dataPw = makeId();\nconst message = \"dataPw += '-x';\";",
        "dataPw"
    ));
    assert!(!has_identifier_reassignment("userid = makeId();", "id"));
    assert!(!has_identifier_reassignment("id => id", "id"));
    assert!(!has_identifier_reassignment("<input id={id}", "id"));
    assert!(!has_identifier_reassignment(
        "<input id={id} data-pw=\"x\">",
        "id"
    ));
    assert!(has_identifier_reassignment("id = {};", "id"));
    assert!(has_identifier_reassignment("id={};", "id"));
    assert!(has_identifier_reassignment("id={foo: 1};", "id"));
}

#[test]
fn enclosing_shadow_binding_requires_an_open_block() {
    assert!(has_enclosing_shadow_binding(
        "function Inner(dataPw) { return <a data-pw={",
        function_param_name_ends("function Inner(dataPw) { return <a data-pw={", "dataPw"),
    ));
    assert!(has_enclosing_shadow_binding(
        "function Inner(dataPw) { if (ready) { dataPw; } return <a data-pw={",
        function_param_name_ends(
            "function Inner(dataPw) { if (ready) { dataPw; } return <a data-pw={",
            "dataPw",
        ),
    ));
    assert!(!has_enclosing_shadow_binding(
        "function Inner(dataPw)",
        function_param_name_ends("function Inner(dataPw)", "dataPw"),
    ));
    assert!(!has_enclosing_shadow_binding(
        "function Inner(dataPw); return <a data-pw={",
        function_param_name_ends("function Inner(dataPw); return <a data-pw={", "dataPw"),
    ));
    assert!(!has_enclosing_shadow_binding(
        "function Inner(dataPw) { return dataPw; } return <a data-pw={",
        function_param_name_ends(
            "function Inner(dataPw) { return dataPw; } return <a data-pw={",
            "dataPw",
        ),
    ));
}

#[test]
fn jsx_start_detection_rejects_comparison_operators() {
    assert!(has_unclosed_jsx_start("<input id={"));
    assert!(has_unclosed_jsx_start("</"));
    assert!(!has_unclosed_jsx_start("if (count <="));
    assert!(!has_unclosed_jsx_start("value <<"));
}

#[test]
fn identifier_shadow_scan_covers_binding_edge_cases() {
    assert!(!prefix_shadows("const", "dataPw"));
    assert!(!prefix_shadows("constdataPw = 1; return ", "dataPw"));
    assert!(!prefix_shadows(
        "function Inner dataPw) { return ",
        "dataPw"
    ));
    assert!(!prefix_shadows("function Inner(dataPw { return ", "dataPw"));
    assert!(!prefix_shadows("", ""));
    assert!(!bindings::is_identifier_at("dataPw", 0, "dataPwx"));
    assert!(!bindings::has_declaration("constx dataPw", "dataPw"));
    assert!(!bindings::has_destructuring_declaration(
        "const dataPw = 1;",
        "dataPw"
    ));
    assert_eq!(
        bindings::function_destructure_binding_ends("function noParen", "dataPw").len(),
        0
    );
    assert_eq!(
        bindings::function_destructure_binding_ends("function Inner({ dataPw", "dataPw").len(),
        0
    );
    assert_eq!(
        bindings::function_destructure_binding_ends("function noParen", "dataPw").len(),
        0
    );
    assert!(!bindings::has_declaration("const", "dataPw"));
    assert!(!bindings::has_destructuring_declaration(
        "const { dataPw",
        "dataPw"
    ));
    assert!(!bindings::has_destructuring_declaration(
        "const { dataPw ;",
        "dataPw"
    ));
    assert_eq!(
        bindings::function_destructure_binding_ends("function Inner({ dataPw )", "dataPw").len(),
        0
    );
    assert_eq!(
        bindings::function_destructure_binding_ends("function Inner()", "").len(),
        0
    );
    assert!(!bindings::is_identifier_at("cafédataPw", 0, "dataPw"));
    assert_eq!(
        bindings::function_destructure_binding_ends("function café({ dataPw }) {", "dataPw").len(),
        1
    );
    assert!(!has_unclosed_jsx_start("{foo"));
    assert_eq!(matching_close_brace("{foo"), None);
    assert_eq!(matching_close_brace("{foo{bar}"), None);
}

#[test]
fn identifier_shadow_scan_matches_declaration_and_function_destructure() {
    assert!(prefix_shadows("const dataPw = 'x'; return ", "dataPw"));
    assert!(prefix_shadows("let dataPw\n", "dataPw"));
    assert!(prefix_shadows("var dataPw ", "dataPw"));
    assert!(!prefix_shadows("constantly dataPw ", "dataPw"));
    assert!(!prefix_shadows("xdataPw ", "dataPw"));
    assert!(prefix_shadows(
        "const { dataPw } = props; return ",
        "dataPw"
    ));
    assert!(prefix_shadows("let [dataPw] = items; return ", "dataPw"));
    assert!(!prefix_shadows(
        "const { dataPwx } = props; return ",
        "dataPw"
    ));
    assert!(prefix_shadows(
        "function Inner({ dataPw }) { return <a data-pw={",
        "dataPw",
    ));
    assert!(prefix_shadows(
        "function Inner([dataPw]) { return <a data-pw={",
        "dataPw",
    ));
    assert!(!prefix_shadows(
        "function helper(dataPw) { return dataPw; } return ",
        "dataPw",
    ));
    assert!(!prefix_shadows(
        "function Inner({ dataPw }) { return dataPw; } return ",
        "dataPw",
    ));
}

fn prefix_shadows(prefix: &str, name: &str) -> bool {
    let source = format!("{prefix}{name}");
    let start = prefix.len() as u32;
    identifier_may_be_shadowed_or_reassigned(
        name,
        Span::new(start, start + name.len() as u32),
        Span::new(0, source.len() as u32),
        &source,
    )
}

fn function_param_name_ends(prefix: &str, name: &str) -> Vec<usize> {
    let mut ends = Vec::new();
    let mut search = 0;
    while let Some(rel) = prefix[search..].find("function") {
        let at = search + rel;
        if !bindings::is_identifier_at(prefix, at, "function") {
            search = at + 1;
            continue;
        }
        let after_fn = &prefix[at + "function".len()..];
        let Some(paren) = after_fn.find('(') else {
            search = at + 1;
            continue;
        };
        let params = &after_fn[paren + 1..];
        let params_end = params.find(')').unwrap_or(params.len());
        let params_region = &params[..params_end];
        if let Some(name_rel) = params_region.find(name) {
            let name_at = at + "function".len() + paren + 1 + name_rel;
            if bindings::is_identifier_at(prefix, name_at, name) {
                ends.push(name_at + name.len());
            }
        }
        search = at + 1;
    }
    ends
}
