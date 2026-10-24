use super::*;

#[test]
fn yaml_flow_quoted_and_next_line_uses_are_checked() {
    // The tag on the last line of flow-pass proves the document parsed, so the
    // pinned, local, parent, and Docker flow entries were judged, not skipped.
    let flow = check_yaml_shape("flow-fail.yml");
    assert_eq!(lines_of(&flow), vec![1], "{flow:?}");
    assert!(flow[0].message.contains("actions/checkout@v4"));
    assert!(flow[0].message.contains("not a 40-char commit SHA"));

    let next_line = check_yaml_shape("next-line-fail.yml");
    assert_eq!(lines_of(&next_line), vec![2], "{next_line:?}");
    assert!(next_line[0].message.contains("actions/checkout@v4"));

    let quoted = check_yaml_shape("quoted-key-fail.yml");
    assert_eq!(lines_of(&quoted), vec![1], "{quoted:?}");
    assert!(quoted[0].message.contains("not a 40-char commit SHA"));

    let passed = check_yaml_shape("flow-pass.yml");
    assert_eq!(lines_of(&passed), vec![8], "{passed:?}");
    assert!(passed[0].message.contains("actions/checkout@v4"));
    assert!(!passed[0].message.contains("de0fac2e"));

    let next_pass = check_yaml_shape("next-line-pass.yml");
    assert_eq!(lines_of(&next_pass), vec![3], "{next_pass:?}");

    let multi = check_yaml_shape("multi-doc-fail.yml");
    assert_eq!(lines_of(&multi), vec![1, 3], "{multi:?}");

    let dedup = check_yaml_shape("dedup-fail.yml");
    assert_eq!(lines_of(&dedup), vec![1, 2], "{dedup:?}");

    // The pinned block uses is already judged. Repeating its ref in run, name,
    // or a comment must not move the finding. The later flow uses still fails.
    let echoed = check_yaml_shape("echo-ref-fail.yml");
    assert_eq!(lines_of(&echoed), vec![5], "{echoed:?}");
    assert!(echoed[0].message.contains("actions/checkout@v4"));
    assert!(!echoed[0].message.contains("de0fac2e"));

    // `from_str` rejects the whole file. Document 1's flow uses must still fail.
    let multi_flow = check_yaml_shape("multi-doc-flow-fail.yml");
    assert_eq!(lines_of(&multi_flow), vec![1], "{multi_flow:?}");
    assert!(multi_flow[0].message.contains("actions/checkout@v4"));

    let folded = check_yaml_shape("folded-flow-fail.yml");
    assert_eq!(lines_of(&folded), vec![1], "{folded:?}");
    assert!(folded[0].message.contains("actions/checkout@v4"));

    let anchored = check_yaml_shape("anchor-flow-fail.yml");
    assert_eq!(lines_of(&anchored), vec![1], "{anchored:?}");
    assert!(anchored[0].message.contains("actions/checkout@v4"));
    assert!(!anchored[0].message.contains("&checkout"));

    let input = check_yaml_shape("with-uses-pass.yml");
    assert_eq!(lines_of(&input), Vec::<usize>::new(), "{input:?}");

    // The input comment shares the resolved ref. The later action still needs
    // its own version comment.
    let stolen = check_yaml_shape("with-comment-fail.yml");
    assert_eq!(lines_of(&stolen), vec![5], "{stolen:?}");
    assert!(stolen[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(stolen[0].message.contains("trailing comment"));

    let alias = check_yaml_shape("alias-fail.yml");
    assert_eq!(lines_of(&alias), vec![3], "{alias:?}");
    assert!(alias[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(alias[0].message.contains("trailing comment"));
    assert!(!alias[0].message.contains("*checkout"));

    let block_input = check_yaml_shape("with-block-uses-pass.yml");
    assert_eq!(
        lines_of(&block_input),
        Vec::<usize>::new(),
        "{block_input:?}"
    );

    let nested = check_yaml_shape("nested-uses-pass.yml");
    assert_eq!(lines_of(&nested), Vec::<usize>::new(), "{nested:?}");

    // `{ uses:` with the ref on the next line still fails.
    let broken = check_yaml_shape("flow-break-fail.yml");
    assert_eq!(lines_of(&broken), vec![3], "{broken:?}");
    assert!(broken[0].message.contains("actions/checkout@v4"));

    // A flow `with` broken across lines is an input. Only the unpinned action fails.
    let flow_input = check_yaml_shape("flow-with-break-fail.yml");
    assert_eq!(lines_of(&flow_input), vec![11], "{flow_input:?}");
    assert!(flow_input[0].message.contains("actions/checkout@v4"));
    assert!(!flow_input[0].message.contains("de0fac2e"));

    // Two uses values on one line both stay paired. The tag is the finding.
    let same_line = check_yaml_shape("same-line-uses-fail.yml");
    assert_eq!(lines_of(&same_line), vec![2], "{same_line:?}");
    assert!(same_line[0].message.contains("actions/checkout@v4"));
    assert!(!same_line[0].message.contains("de0fac2e"));

    // A run script is not an action line. The block uses value still fails.
    let script = check_yaml_shape("block-script-fail.yml");
    assert_eq!(lines_of(&script), vec![8, 11], "{script:?}");
    assert!(script
        .iter()
        .all(|finding| finding.message.contains("actions/checkout@v4")));
    assert!(script
        .iter()
        .all(|finding| !finding.message.contains("de0fac2e")));

    // The matrix comment must not validate the step, and include is not a finding.
    let matrix = check_yaml_shape("matrix-uses-fail.yml");
    assert_eq!(lines_of(&matrix), vec![9, 17], "{matrix:?}");
    assert!(matrix[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(matrix[0].message.contains("trailing comment"));
    assert!(!matrix[0].message.contains("ubuntu-latest"));
    assert!(matrix[1].message.contains("actions/checkout@v4"));
    assert!(!matrix[1].message.contains("ubuntu-latest"));

    // `\"` inside the name is not the end of the scalar.
    let escaped = check_yaml_shape("quote-escape-fail.yml");
    assert_eq!(lines_of(&escaped), vec![2], "{escaped:?}");
    assert!(escaped[0].message.contains("actions/checkout@v4"));

    // A comment on `|` must not move the step finding onto the script line.
    let header = check_yaml_shape("block-header-comment-fail.yml");
    assert_eq!(lines_of(&header), vec![4, 7], "{header:?}");
    assert!(header
        .iter()
        .all(|finding| finding.message.contains("actions/checkout@v4")));
    assert!(header
        .iter()
        .all(|finding| !finding.message.contains("de0fac2e")));

    // `uses :` with a space before the colon is still an action key.
    let spaced = check_yaml_shape("space-before-colon-fail.yml");
    assert_eq!(lines_of(&spaced), vec![2, 3], "{spaced:?}");
    assert!(spaced[0].message.contains("actions/checkout@v4"));
    assert!(spaced[1].message.contains("actions/checkout@v3"));

    // `#` after an escaped quote stays inside the name. The pinned line passes.
    let comment = check_yaml_shape("escaped-comment-fail.yml");
    assert_eq!(lines_of(&comment), vec![3], "{comment:?}");
    assert!(comment[0].message.contains("actions/checkout@v4"));
    assert!(!comment[0].message.contains("trailing comment"));
    assert!(!comment[0].message.contains("de0fac2e"));

    // The alias line is the step. The anchor's tag must not be dropped.
    let alias_step = check_yaml_shape("mapping-alias-fail.yml");
    assert_eq!(lines_of(&alias_step), vec![8, 16], "{alias_step:?}");
    assert!(alias_step[0].message.contains("actions/checkout@v4"));
    assert!(alias_step[1].message.contains("actions/checkout@v3"));

    // `\u0065` in the quoted key is `e`, so the unpinned action is still found.
    let encoded = check_yaml_shape("quoted-escape-key-fail.yml");
    assert_eq!(lines_of(&encoded), vec![2], "{encoded:?}");
    assert!(encoded[0].message.contains("actions/checkout@v4"));

    // `foo#bar` is one plain scalar. The unpinned action after it still fails.
    let hashed = check_yaml_shape("plain-hash-fail.yml");
    assert_eq!(lines_of(&hashed), vec![2], "{hashed:?}");
    assert!(hashed[0].message.contains("actions/checkout@v4"));

    // The header comment covers the pinned block value. The bare header does not.
    let chomp = check_yaml_shape("block-chomp-comment-fail.yml");
    assert_eq!(lines_of(&chomp), vec![6, 9], "{chomp:?}");
    assert!(chomp[0].message.contains("trailing comment"));
    assert!(chomp[0].message.contains("de0fac2e"));
    assert!(chomp[1].message.contains("actions/checkout@v4"));
    assert!(!chomp[1].message.contains("trailing comment"));
}

#[test]
fn same_column_and_bare_dash_steps_are_checked() {
    // A dash in the steps column is still a step. The include field is not.
    let same = check_yaml_shape("same-indent-steps-fail.yml");
    assert_eq!(lines_of(&same), vec![10, 11], "{same:?}");
    assert!(same[0].message.contains("actions/checkout@v4"));
    assert!(same[1].message.contains("actions/setup-node@v4"));

    // The key on the line after a bare dash, including a commented dash, is the step.
    let bare = check_yaml_shape("bare-dash-step-fail.yml");
    assert_eq!(lines_of(&bare), vec![6, 8], "{bare:?}");
    assert!(bare[0].message.contains("actions/checkout@v4"));
    assert!(bare[1].message.contains("actions/setup-node@v4"));
}

#[test]
fn anchor_only_and_plain_apostrophe_steps_are_checked() {
    let anchored = check_yaml_shape("anchor-step-fail.yml");
    assert_eq!(lines_of(&anchored), vec![6, 7], "{anchored:?}");
    assert!(anchored[0].message.contains("actions/checkout@v4"));
    assert!(anchored[1].message.contains("actions/setup-node@v4"));

    // The apostrophe stays inside the plain name, so the next flow step fails.
    let named = check_yaml_shape("plain-apostrophe-fail.yml");
    assert_eq!(lines_of(&named), vec![3], "{named:?}");
    assert!(named[0].message.contains("actions/checkout@v4"));

    // The apostrophe stays in the name, so the trailing version comment counts.
    let comment = check_yaml_shape("plain-apostrophe-comment-fail.yml");
    assert_eq!(lines_of(&comment), vec![4], "{comment:?}");
    assert!(comment[0].message.contains("trailing comment"));
    assert!(comment[0].message.contains("de0fac2e"));
}
