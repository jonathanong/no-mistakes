<!-- cspell:ignore toplevel -->

# `no-mistakes tests audit`

Compare a saved targeted plan with execution observations from a full-suite run.
This stateless command reads two artifacts; it does not run tests, discover files,
rebuild the graph, persist coverage, or verify the current checkout.

```sh
no-mistakes tests audit --plan /tmp/audit-plan.json --observations /tmp/audit-run.json
no-mistakes tests audit --plan /tmp/audit-plan.json --observations /tmp/audit-run.json --format text
```

`--plan` and `--observations` are required. `--format json|text` defaults to
`json`. Successful audits exit zero even when observed misses are found; CI
callers can enforce their own policy using `missed_observed_tests`.

## Artifact contract, version 1

The plan envelope contains `schema_version: 1`, `provenance`, `plan` (the unchanged
JSON from `tests plan` or `tests impact`), `changed_files`, and `changed_symbols`.
`changed_files` must explicitly identify at least one changed file and match the
embedded plan inventory when that inventory is nonempty. An impact plan may have
an empty inventory; its envelope still requires the actual changed-file scope.
`changed_symbols` is an array of `{ file, symbol }`. For a file listed there,
only an exact symbol match counts as execution of the change; other changed
files match any execution of that file. Use an empty array for file-level audits.
Symbol IDs must have the same meaning in the planner input and trace adapter:
do not guess exported IDs from runtime function names or line ranges.

Both artifacts require identical `provenance`:

- `checkout_revision`: lowercase 40- or 64-digit Git revision.
- `source_digest`: lowercase 64-digit SHA-256 identity of the complete source and
  configuration snapshot, including dirty files when the producer supports them.
- `scope_digest`: lowercase 64-digit SHA-256 identity of the full-suite
  runner/project/configuration/instrumentation scope.

These identities are **caller-supplied metadata**. Matching them prevents accidental
comparison across declared snapshots/scopes; the audit does not recompute or
cryptographically verify them. A revision alone cannot identify dirty sources.
Include external configs, generated inputs, and runtime dependencies in the
producer's snapshot/scope when they affect execution.

The observations artifact contains `schema_version: 1`, the same `provenance`,
`granularity: "per-test-file"`, `suite: "full"`, `complete: true`, and `tests`.
Each test record has `test_file`, `executed_files`, `executed_symbols` (the same
`{ file, symbol }` shape), and `trace_complete`. Every executed symbol's file
must also appear in `executed_files`. `complete` declares that the full-suite run
finished; `trace_complete` separately declares complete tracing for this file
within the declared file/symbol scope. Set it to false for dropped events,
incomplete instrumentation, early interruption, or unknown trace coverage.

One observation record represents one test file. Merge all case, retry, and
project executions within the declared scope before producing that record;
union execution hits and mark the merged trace incomplete if any constituent
trace is incomplete. Duplicate test-file records and selected test files are
rejected. Duplicate execution hits are deduplicated. All paths must be normalized
root-relative slash paths: absolute paths, backslashes, drive prefixes, empty
components, `.`/`..`, and control characters are rejected. Unknown envelope and
observation fields are rejected.

Whole-suite aggregate Istanbul/V8 coverage cannot identify which test file
executed a changed file. It is unsupported input, even if every test passed.

## Producer recipe: isolated Istanbul coverage per test file

This example wraps a real `testsPlan()` result and adapts Istanbul JSON maps
collected **separately for each test file** during a completed full-suite run.
Use a clean immutable Git checkout for this recipe and store artifacts outside
the checkout. Configure instrumentation to cover the relevant source scope; if
its completeness is unknown, supply `traceComplete: false`. The producer must
record the same identities when planning and when running the suite, reject
source/scope changes between those stages, and never relabel older traces.

A runner adapter supplies `/tmp/full-suite-coverage.json`:

```json
{
  "suite": "full",
  "complete": true,
  "provenance": {
    "checkoutRevision": "<revision captured before suite execution>",
    "sourceDigest": "<snapshot digest captured before suite execution>",
    "scopeDigest": "<scope digest captured before suite execution>"
  },
  "tests": [
    {
      "testFile": "tests/api.test.mts",
      "coverageMap": "/tmp/coverage/api/coverage-final.json",
      "traceComplete": true
    }
  ]
}
```

The `tests` list must cover the runner's complete configured suite inventory,
including files with zero execution hits. Each `coverageMap` is that test file's
isolated Istanbul map, not a copy of the same aggregate map. Execution hits are
positive statement, function, or branch counters; listed-but-unexecuted files do
not count.
The following executable Node example produces camelCase artifacts, accepted by
`testsAudit()`. CLI artifact JSON uses the equivalent snake_case names.

```js
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { isAbsolute, relative, resolve } from 'node:path';
import { testsPlan, testsAudit } from 'no-mistakes';

const root = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
const git = (...args) => execFileSync('git', ['-C', root, ...args]);
const sha256 = value => createHash('sha256').update(value).digest('hex');
const capture = () => {
  if (git('status', '--porcelain', '--untracked-files=all').length) {
    throw new Error('This recipe requires a clean source checkout');
  }
  return {
    checkoutRevision: git('rev-parse', 'HEAD').toString().trim(),
    sourceDigest: sha256(git('ls-tree', '-rz', '--full-tree', 'HEAD')),
    // Share this exact scope definition with the runner adapter.
    scopeDigest: sha256(JSON.stringify({ runner: 'vitest', config: 'vitest.config.mts',
      projects: ['unit'], instrumentation: 'istanbul', mode: 'isolated-test-file' })),
  };
};
const sameProvenance = (left, right) => ['checkoutRevision', 'sourceDigest', 'scopeDigest']
  .every(key => left?.[key] === right[key]);
const provenance = capture();
const plan = await testsPlan({ root, framework: 'vitest', changedFiles: ['src/api.mts'] });
const planJson = { schemaVersion: 1, provenance, plan,
  changedFiles: plan.changedFiles, changedSymbols: [] };
const run = JSON.parse(await readFile('/tmp/full-suite-coverage.json', 'utf8'));
if (run.suite !== 'full' || run.complete !== true ||
    !sameProvenance(run.provenance, provenance) ||
    !sameProvenance(capture(), provenance)) {
  throw new Error('Run provenance/scope changed or the full suite did not finish');
}
const tests = await Promise.all(run.tests.map(async row => {
  const coverage = JSON.parse(await readFile(row.coverageMap, 'utf8'));
  const executedFiles = Object.values(coverage)
    .filter(file => [...Object.values(file.s), ...Object.values(file.f),
      ...Object.values(file.b).flat()].some(hits => hits > 0))
    .map(file => {
      const path = relative(root, resolve(root, file.path)).replaceAll('\\', '/');
      if (isAbsolute(path) || path === '..' || path.startsWith('../')) {
        throw new Error('Coverage source is outside the snapshot');
      }
      return path;
    });
  return { testFile: row.testFile, executedFiles, executedSymbols: [],
    traceComplete: row.traceComplete === true };
}));
const observationsJson = { schemaVersion: 1, provenance,
  granularity: 'per-test-file', suite: 'full', complete: true, tests };
if (!sameProvenance(capture(), provenance)) throw new Error('Source snapshot changed');
// Save CLI-compatible artifacts, or pass the camelCase objects directly to the API.
const snake = value => Array.isArray(value) ? value.map(snake) :
  value && typeof value === 'object' ? Object.fromEntries(Object.entries(value)
    .map(([key, nested]) => [key.replace(/[A-Z]/g, c => '_' + c.toLowerCase()), snake(nested)])) : value;
await writeFile('/tmp/audit-plan.json', JSON.stringify(snake(planJson)));
await writeFile('/tmp/audit-run.json', JSON.stringify(snake(observationsJson)));
const report = await testsAudit({ planJson, observationsJson });
console.log(JSON.stringify(report, null, 2));
```

The recipe audits files. Symbol audits additionally require a trace adapter that
emits the exact symbol identities agreed by the planner input and trace adapter,
and declares tracing complete for those symbols. Do not mark statement-only
coverage complete for a symbol audit.

## Reading results

`missed_observed_tests` lists unselected test files that demonstrably executed
changed files or symbols, with `matched_files` and `matched_symbols` evidence.
`selected_observed_tests` gives the same evidence for selected files.
`selected_without_observed_execution` lists complete traces with no matching
execution, preserving static plan `reasons` for investigation.
`selected_without_observations` and `selected_with_incomplete_traces` represent
unknown evidence, not excess selection. Incomplete traces still contribute
positive hits. Output ordering is deterministic.

No observed misses **does not prove completeness**. Unexecuted branches,
uninstrumented code, indirect effects, and changed behavior remain outside the
observations. Negative evidence does not prove that a selected test is unnecessary.
Passing test outcomes provide no completeness proof. This comparison is at
file granularity, not individual test-case or runner-project granularity.
Every report includes these limitations.

Node API: `testsAudit({ plan, observations })` or
`testsAudit({ planJson, observationsJson })`. Each input accepts exactly one path
or artifact object/JSON text. The native async worker reads files, parses JSON
text, and normalizes snake_case/camelCase artifact keys; the facade preserves
file paths and text without decoding them on the JavaScript event loop. Results
use camelCase. `analyzeProject()` supports
`{ type: 'testsAudit', planJson, observationsJson }` and the same path options.
