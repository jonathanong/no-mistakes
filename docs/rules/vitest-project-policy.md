# `vitest-project-policy`

## Why and when

Keep explicitly declared Vitest projects consistent without importing live config
or making a test scan the checkout. This opt-in rule reads TypeScript/JavaScript
config source and checks literal properties of each `test.projects[]` entry.
It never executes config. Only `config` and the finite `helpers` list are inputs;
it does not discover additional config files, expand project globs, or scan tests.

## What it catches

The rule checks required scalar values, forbidden declarations, paired property
presence, and corresponding values across the explicitly declared projects.

## Configuration

```yaml
rules:
  - rule: vitest-project-policy
    scope: repository
    options:
      config: vitest.config.ts
      helpers: [config/test-projects.ts]
      required:
        isolate: true
      forbidden: [poolOptions]
      parity:
        - properties: [fileParallelism, maxWorkers]
          compareRoot: false
```

Paths are relative to the repository root. `config` is required. `helpers`
defaults to an empty list. `required`, `forbidden`, and `parity` default to empty
policies. Property names are direct keys of each project's `test` object.
`required` maps names to exact scalar literals: strings, numbers, booleans, or
null. `forbidden` requires a property to be absent, regardless of its value.

Each `parity` entry names exactly two properties. A project that declares either
must declare both. Corresponding values must agree across projects, using the
first complete project's pair as the baseline. The values of the two different
properties need not equal one another: `fileParallelism: false` and
`maxWorkers: 1` form a valid pair. Projects with neither property are allowed.
With `compareRoot: true`, each corresponding value instead compares with the
root config's `test` object; a missing or unresolved root value is incomplete.

The shared `message` field replaces finding text. Normal severity and
`include`/`exclude` application filters apply. There is no automatic fix: choosing
worker settings or changing inheritance can change how tests run.

## Valid example

```ts
import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    projects: [
      { test: { name: 'unit', isolate: true, fileParallelism: false, maxWorkers: 1 } },
      { test: { name: 'integration', isolate: true, fileParallelism: false, maxWorkers: 1 } },
    ],
  },
});
```

## Counterexample

The second project omits one member of the configured pair:

```ts
import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    projects: [
      { test: { name: 'unit', fileParallelism: false, maxWorkers: 1 } },
      { test: { name: 'integration', fileParallelism: false } },
    ],
  },
});
```

## Fix

Declare `maxWorkers: 1` on the second project's test options. For a mismatch,
choose a common literal value for that property across the configured projects.
For required or forbidden settings, add the required literal or remove the
forbidden declaration. Move shared literal objects or arrays into a listed
helper and import them statically to keep the source policy reviewable.

## Analysis and uncertainty

The rule reuses the Vitest config parser's lexical utilities and the request's
source store and parsed-program cache. Static object/array literals, local
constant bindings, supported config wrappers, explicit helper imports, and
object spreads preserve source locations. `extends: true` inherits root test
settings. Root values are otherwise used for comparison only when requested.

Non-literal property values, unresolved project entries, unknown spreads,
unsupported calls, missing files, and imports outside `helpers` appear in
`CheckReport.incomplete` with the rule consumer, file, property when known, and
rewrite guidance. They are never guessed or treated as a passing literal.
Use literal properties and explicit helper inputs to make the policy complete.
Assignments, updates, or deletes in a declared source conservatively make its
config incomplete. The policy checks source declarations; it does not model
side effects of unrelated calls or imports, or certify runtime configuration
after such effects. Rewrite mutation and computed configuration as literal
declarations when enforcing effective runtime settings.

## Suppression

Use `// no-mistakes-disable-file vitest-project-policy` to suppress a whole
source file, or `no-mistakes-disable-line` / `no-mistakes-disable-next-line` for
a finding's reported line. A missing property is reported on its containing
project object; a present property's diagnostic points at that property.

Suppression caveat: put directives in the source file and at the location shown
by the report, including helper files. A comment in an importing file does not
suppress a finding located in its helper. Incomplete prerequisite evidence is
separate from suppressible findings.

## Node API

The same configured rule runs through asynchronous `check()` and batched
`analyzeProject()` checks. Exported `VitestProjectPolicyOptions`,
`VitestPropertyParity`, and `VitestPropertyLiteral` describe rule configuration.
The existing async `checkJson` native binding and JS facade return ordinary
findings plus `CheckReport.incomplete`; no additional function is needed.


## Related rules

[`vitest-project-mapping`](vitest-project-mapping.md) checks declared project
coverage. [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks CI path
selection. [`test-no-live-repository-scan`](test-no-live-repository-scan.md)
keeps tests from scanning live checkout inputs; this rule can replace a test
that imports config solely to enforce a static project property policy.
