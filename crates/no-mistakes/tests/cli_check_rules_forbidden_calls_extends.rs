#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

use std::process::Command;

/// A class base is not a call. With `unconstructed-error-class` configured the
/// shared graph carries `extends` edges, and `forbidden-calls` must still read
/// only calls: `Child extends Base` never matches a target naming `Base`, while
/// `new Base()` does.
#[test]
fn an_extends_clause_never_matches_a_forbidden_call_target() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "extends-is-not-a-call");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "human", "--root"])
        .arg(root.path())
        .arg("--config")
        .arg(root.path().join(".no-mistakes.yml"))
        .output()
        .unwrap();
    let body = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut lines: Vec<_> = body.lines().collect();
    lines.sort_unstable();

    assert_eq!(output.status.code(), Some(1), "{body}");
    // Only the `new Base()` in `src/direct.ts` is reported by each application:
    // nothing points at `src/child.ts`, whose two classes extend `Base`.
    assert_eq!(
        lines,
        [
            "forbidden-calls src/direct.ts:4 forbidden call (construct-target, application #2): \
             exact `Base`",
            "forbidden-calls src/direct.ts:4 forbidden call (function-target, application #1): \
             repository function `src/base.ts#Base`",
            // The same run resolved the hierarchy: `Orphan` reaches `Error`
            // through `Base` in another file.
            "unconstructed-error-class src/child.ts:6 exported error class `Orphan` is never \
             constructed or subclassed in non-test source",
        ],
        "{body}"
    );
}
