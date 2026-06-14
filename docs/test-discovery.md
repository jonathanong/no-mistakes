# Test Discovery

Test discovery has one ownership rule: configured project include/exclude globs
own discovery before any generic runner fallback runs.

## Invariants

- Configured `include` and `exclude` globs are evaluated first for each runner
  project.
- A file included by a configured project is project-scoped even when that same
  project excludes it.
- Excluded configured files must not be revived later by generic fallback.
- Runner fallback is a last resort. It may select only test-shaped files such as
  `*.test.*`, `*.spec.*`, or files under `__tests__/`.
- Fallback does not infer project-specific conventions. It uses explicit runner
  heuristics only after configured projects and cross-runner reservations are
  applied.
- Playwright top-level config `name` is policy metadata only. It is not a
  Playwright CLI `--project` value.
- `TestExecutionTarget` describes executable command metadata: runner, config,
  optional executable project argument, base command, and runner arguments.

## Internal Project Metadata

`ConfigProject.policy_name` is the stable name used for matching configured
policies. `ConfigProject.runner_project_arg` is the executable project name that
may become `--project <name>` in a runner command.

Vitest project names populate both fields. Playwright `projects[].name`
populates both fields. Playwright top-level config `name` populates only
`policy_name`, because Playwright does not accept it as a CLI project selector.
Policy-only projects set both fields to the policy key.


## Swift

Swift discovery is explicit and package-scoped. Configure SwiftPM package roots
with `tests.swift.packages`; each `Package.swift` contributes discovered
`.testTarget(...)` targets under `Tests/<target>/**/*.swift`. Targets run as
`swift test --package-path <package> --filter <test-target>`.
