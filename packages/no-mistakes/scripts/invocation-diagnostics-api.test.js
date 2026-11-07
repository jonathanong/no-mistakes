const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-excluded-roots",
);

function stable({ jobsPoolWarning: _warning, invocationDiagnostics: _diagnostics, ...report }) {
  return report;
}

function annotation(diagnostics) {
  const work = diagnostics.work;
  assert.equal(work["postgres.annotation.roots_before_scoping"], 2);
  assert.equal(work["postgres.annotation.roots_after_scoping"], 1);
  assert.equal(work["postgres.annotation.module_initializations"], 1);
  assert.equal(work["postgres.annotation.shared_init.disabled"], 1);
  for (const name of ["cache_hits", "cache_misses", "builds"]) {
    assert.equal(work[`postgres.annotation.shared_init.${name}`], 0);
  }
  assert.ok(
    diagnostics.timings.some(
      (span) => span.label === "postgres.annotation.shared_init.build" && span.durationMs === 0,
    ),
  );
  for (const span of diagnostics.timings) {
    assert.equal(typeof span.label, "string");
    assert.equal(typeof span.durationMs, "number");
    assert.equal(typeof span.parallel, "boolean");
  }
}

test(
  "compiled verbose invocation diagnostics preserve default reports and async CLI parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const command = spawnSync(cli, ["check", "--root", root, "--format", "json", "--jobs", "1"], {
      encoding: "utf8",
    });
    assert.equal(command.status, 1, command.stderr);
    const cliReport = JSON.parse(command.stdout);
    for (const api of [cjs, esm]) {
      const plain = await api.check({ root, jobs: 1 });
      assert.equal(Object.hasOwn(plain, "invocationDiagnostics"), false);
      assert.deepEqual(stable(plain), stable(cliReport));
      assert.deepEqual(
        stable(await api.check({ root, jobs: 1, diagnosticsVerbose: false })),
        stable(plain),
      );
      const id = "compiled-verbose-invocation";
      const pending = api.check({ root, jobs: 1, diagnosticsVerbose: true, diagnosticsId: id });
      assert.ok(pending instanceof Promise);
      const verbose = await pending;
      annotation(verbose.invocationDiagnostics);
      assert.deepEqual(stable(verbose), stable(plain));
      assert.equal(await api.diagnosticsSnapshot({ id }), null);
      const batch = await api.analyzeProject({
        root,
        jobs: 1,
        diagnosticsVerbose: true,
        reports: [{ type: "check" }],
      });
      annotation(batch.invocationDiagnostics);
      assert.deepEqual(stable(batch.reports[0].result), stable(plain));
      assert.equal(Object.hasOwn(batch.reports[0].result, "invocationDiagnostics"), false);
      await assert.rejects(
        api.check({ root, diagnosticsVerbose: "true" }),
        /diagnosticsVerbose must be a boolean/u,
      );
      await assert.rejects(
        api.check({ root, diagnosticsId: id, diagnosticsVerbose: true, unknownOption: true }),
      );
      assert.equal(await api.diagnosticsSnapshot({ id }), null);
    }
  },
);
