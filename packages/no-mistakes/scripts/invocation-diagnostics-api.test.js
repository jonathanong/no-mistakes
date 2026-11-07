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

function stable({ jobsPoolWarning: _warning, ...report }) {
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
  assert.equal(typeof diagnostics.elapsedMs, "number");
  if (process.platform === "win32") assert.deepEqual(diagnostics.phases, []);
  else assert.ok(diagnostics.phases.length > 0);
  for (const phase of diagnostics.phases) {
    assert.equal(typeof phase.label, "string");
    for (const field of [
      "wallMs",
      "userMs",
      "systemMs",
      "utilization",
      "peakRssKiB",
      "peakRssDeltaKiB",
    ]) {
      assert.equal(typeof phase[field], "number", field);
    }
  }
  assert.ok(diagnostics.hotFiles.some((file) => file.pass === "postgres.annotation"));
  for (const file of diagnostics.hotFiles) {
    assert.equal(typeof file.pass, "string");
    assert.equal(typeof file.path, "string");
    assert.equal(typeof file.durationMs, "number");
    assert.equal(typeof file.units, "number");
  }
  for (const span of diagnostics.timings) {
    assert.equal(typeof span.label, "string");
    assert.equal(typeof span.durationMs, "number");
    assert.equal(typeof span.parallel, "boolean");
  }
}

test(
  "compiled annotation collectDiagnostics preserves default reports and async CLI parity",
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
      assert.deepEqual(stable(plain), stable(cliReport));
      const id = "compiled-annotation-diagnostics";
      const pending = api.collectDiagnostics("check", { root, jobs: 1, diagnosticsId: id });
      assert.ok(pending instanceof Promise);
      const observed = await pending;
      annotation(observed.diagnostics);
      assert.deepEqual(stable(observed.result), stable(plain));
      assert.equal(await api.diagnosticsSnapshot({ id }), null);
      const batch = await api.collectDiagnostics("analyzeProject", {
        root,
        jobs: 1,
        reports: [{ type: "check" }],
      });
      annotation(batch.diagnostics);
      assert.deepEqual(stable(batch.result.reports[0].result), stable(plain));
      assert.equal(Object.hasOwn(batch.result, "diagnostics"), false);
      await assert.rejects(
        api.collectDiagnostics("check", { root, diagnosticsId: id, unknownOption: true }),
        (error) => {
          assert.ok(error.diagnostics);
          assert.ok(Array.isArray(error.diagnostics.phases));
          return true;
        },
      );
      assert.equal(await api.diagnosticsSnapshot({ id }), null);
      assert.deepEqual(stable(await api.check({ root, jobs: 1 })), stable(plain));
    }
  },
);
