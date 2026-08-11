use super::*;

#[test]
fn direct_pull_request_actions_are_correlated_with_job_conditions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/configured.yml",
                "on:\n  pull_request:\n    types: [closed, synchronize]\njobs:\n  closed:\n    if: github.event.action == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p closed/tsconfig.json\n  bracketed-closed:\n    if: github.event['action'] == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p bracketed-closed/tsconfig.json\n  nested-bracketed-closed:\n    if: github['event']['action'] == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p nested-bracketed-closed/tsconfig.json\n  synchronize:\n    if: github.event.action == 'synchronize'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p synchronize/tsconfig.json\n",
            ),
            document(
                ".github/workflows/default.yml",
                "on: pull_request\njobs:\n  closed:\n    if: github.event.action == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p default-closed/tsconfig.json\n  synchronize:\n    if: github.event.action == 'synchronize'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p default-synchronize/tsconfig.json\n",
            ),
            document(
                ".github/workflows/event-isolation.yml",
                "on:\n  push:\n  pull_request:\n    types: [closed]\njobs:\n  closed:\n    if: github.event.action == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p isolated-closed/tsconfig.json\n  push:\n    if: github.event_name == 'push'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "closed/tsconfig.json",
        "bracketed-closed/tsconfig.json",
        "nested-bracketed-closed/tsconfig.json",
        "synchronize/tsconfig.json",
        "default-closed/tsconfig.json",
        "default-synchronize/tsconfig.json",
        "isolated-closed/tsconfig.json",
        "push/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "default-synchronize/tsconfig.json".to_string(),
            "push/tsconfig.json".to_string(),
            "synchronize/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn direct_event_action_truthiness_keeps_push_and_pull_request_activations_isolated() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/push.yml",
                "on: push\njobs:\n  dot-positive:\n    if: github.event.action\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-dot-positive/tsconfig.json\n  bracket-positive:\n    if: github.event['action']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-bracket-positive/tsconfig.json\n  nested-bracket-positive:\n    if: github['event']['action']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-nested-bracket-positive/tsconfig.json\n  dot-negated:\n    if: '!github.event.action'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-dot-negated/tsconfig.json\n  bracket-negated:\n    if: \"!github.event['action']\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-bracket-negated/tsconfig.json\n  nested-bracket-negated:\n    if: \"!github['event']['action']\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-nested-bracket-negated/tsconfig.json\n",
            ),
            document(
                ".github/workflows/pull-request.yml",
                "on:\n  pull_request:\n    types: [synchronize]\njobs:\n  dot-positive:\n    if: github.event.action\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-dot-positive/tsconfig.json\n  bracket-positive:\n    if: github.event['action']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-bracket-positive/tsconfig.json\n  nested-bracket-positive:\n    if: github['event']['action']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-nested-bracket-positive/tsconfig.json\n  dot-negated:\n    if: '!github.event.action'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-dot-negated/tsconfig.json\n  bracket-negated:\n    if: \"!github.event['action']\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-bracket-negated/tsconfig.json\n  nested-bracket-negated:\n    if: \"!github['event']['action']\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-nested-bracket-negated/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "push-dot-positive/tsconfig.json",
        "push-bracket-positive/tsconfig.json",
        "push-nested-bracket-positive/tsconfig.json",
        "push-dot-negated/tsconfig.json",
        "push-bracket-negated/tsconfig.json",
        "push-nested-bracket-negated/tsconfig.json",
        "pull-request-dot-positive/tsconfig.json",
        "pull-request-bracket-positive/tsconfig.json",
        "pull-request-nested-bracket-positive/tsconfig.json",
        "pull-request-dot-negated/tsconfig.json",
        "pull-request-bracket-negated/tsconfig.json",
        "pull-request-nested-bracket-negated/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "push-dot-negated/tsconfig.json".to_string(),
            "push-bracket-negated/tsconfig.json".to_string(),
            "push-nested-bracket-negated/tsconfig.json".to_string(),
            "pull-request-dot-positive/tsconfig.json".to_string(),
            "pull-request-bracket-positive/tsconfig.json".to_string(),
            "pull-request-nested-bracket-positive/tsconfig.json".to_string(),
        ])
    );
}
