# `vitest-project-policy`

## Why and when

Keep explicitly declared Vitest projects consistent without importing live config
or making a test scan the checkout. This opt-in rule reads TypeScript/JavaScript
config source and checks literal properties of each `test.projects[]` entry.
It never executes config. Only `config` and the configured `helpers` are inputs;
it does not discover additional config files, expand project globs, or scan tests.

## What it catches

The rule checks required scalar values, forbidden declarations, paired property
presence, and corresponding values across the explicitly declared projects.
Optional `declared` policies also reject literal values or literal kinds anywhere
in object literals in the configured sources, without resolving projects.

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
defaults to an empty list and accepts paths or globs such as
`helpers/vitest-config/*.mts`. Globs expand against the request's prepared
visible-file inventory; they never trigger a filesystem walk. CLI, asynchronous
Node checks, and standalone filesystem dispatch use the same inventory, including
visible helpers under `filesystem.skipDirectories`. These helpers can supply
effective imported values and declaration findings. The configured `config`
must still pass project scope, skip-directory, and include/exclude selection;
helpers cannot reopen an excluded config.

An existing literal helper path takes precedence over glob syntax. For example,
`helpers/[shared]/projects.ts` and `helpers/projects[.ts` select those exact files,
even though their names contain matched or unmatched brackets. This applies to
characters in both filenames and directory components. Remaining entries are
expanded as globs; invalid glob syntax still reports a configuration error.

`required`, `forbidden`, and `parity` default to empty policies. Property names are direct keys of each project's `test` object.
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
// vitest.config.ts
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
// vitest.config.ts
import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    projects: [
      { test: { name: 'unit', isolate: true, fileParallelism: false, maxWorkers: 1 } },
      { test: { name: 'integration', isolate: true, fileParallelism: false } },
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

## Declared literal policies

Use these source restrictions when settings must come from a computed shared
policy rather than a hardcoded literal:

```yaml
options:
  config: vitest.config.ts
  helpers: ["helpers/vitest-config/*.mts"]
  declared:
    forbiddenValues:
      fileParallelism: [false]
    forbidLiteralKinds:
      maxWorkers: [number]
```

Both maps default to empty. `forbiddenValues` accepts arrays of string, number,
boolean, or null literals. `forbidLiteralKinds` accepts arrays containing
`number`, `string`, or `boolean`. These checks inspect every object literal in
`config` and matched helpers: root, project, nested, and unimported helper
objects all count. They also run when project declarations are dynamic.

For this configuration, a valid helper uses a computed value:

```ts
export const workers = { maxWorkers: parseVitestMaxWorkers() };
```

Counterexample:

```ts
export const settings = { fileParallelism: false, 'maxWorkers': 2 };
```

Remove the forbidden assignment or replace its literal with the intended shared
computation. Quoted and unquoted keys match equally. Numeric literals include
negative numbers and separators (`-1`, `1_000`). Comments never match.
Identifiers, calls, and member expressions are allowed; these checks do not
propagate constant bindings, so `maxWorkers: WORKERS` is allowed even if
`WORKERS` is initialized with a number. A computed expression is not judged by
its eventual runtime value.

The declared checks produce findings at the property declaration. With only
`declared` configured, a dynamic project list does not produce incomplete
evidence: these checks inspect declarations without resolving that list.
However, a configured config/helper source that cannot be read or parsed is
reported in `CheckReport.incomplete`, attributed to that source and property
`declared`. This also covers a glob-matched file that disappears after the
prepared inventory was captured. Restore readable source or fix its syntax;
unavailable sources never count as successfully checked. A helper glob only
selects visible files; an unmatched glob does not invent missing source paths.
Rust entrypoints that return only findings return a source error for these
failures rather than discarding the incomplete evidence.

Each rule application's include/exclude filters apply to declaration sources
before scanning. Excluded helpers produce neither declaration findings nor
source-failure evidence for that application. The complete configured helper
set remains available for effective-value resolution, so excluding a helper's
declarations does not break an included project's imported settings.
The existing effective `required`, `forbidden`, and `parity` policies keep
their resolution and uncertainty behavior.

## Analysis and uncertainty

The rule reuses the Vitest config parser's lexical utilities and the request's
source store and parsed-program cache. Static object/array literals, local
constant bindings, supported config wrappers, explicit helper imports, and
object spreads preserve source locations. Only explicit `extends: true` applies
root inheritance here, excluding [`name`, `projects`, and root `globalSetup`](https://main.vitest.dev/guide/projects#configuration); version-specific inheritance defaults are not inferred.
Root values are otherwise used for comparison only when requested.

For effective `required`, `forbidden`, and `parity` policies, non-literal
property values, unresolved project entries, unknown spreads,
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
suppress a finding located in its helper. A file directive also suppresses incomplete evidence attributed to that source.
A file directive in a readable but malformed helper also suppresses its
source-failure evidence. An unreadable file cannot provide a source directive.
Incomplete evidence has no line, so line directives apply only to findings.

## Node API

The same configured rule runs through asynchronous `check()` and batched
`analyzeProject()` checks. Exported `VitestProjectPolicyOptions`,
`VitestDeclaredProjectPolicy`, `VitestPropertyLiteralKind`,
`VitestPropertyParity`, and `VitestPropertyLiteral` describe rule configuration.
The existing async `checkJson` native binding and JS facade return ordinary
findings plus `CheckReport.incomplete`; no additional function is needed.


## Related rules

[`vitest-project-mapping`](vitest-project-mapping.md) checks declared project
coverage. [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks CI path
selection. [`test-no-live-repository-scan`](test-no-live-repository-scan.md)
keeps tests from scanning live checkout inputs; this rule can replace a test
that imports config solely to enforce a static project property policy.
