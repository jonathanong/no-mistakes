const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { parse } = require("yaml");
const workflow = parse(readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8"));

test("standalone downloads retry failures within the original total budget", () => {
  const job = workflow.jobs["standalone-shards"];
  const downloads = job.steps.filter((step) => step.uses?.startsWith("actions/download-artifact@"));
  assert.equal(downloads.length, 2);
  const [first, retry] = downloads;
  assert.equal(first.id, "standalone-download");
  assert.equal(first["continue-on-error"], true);
  assert.equal(first["timeout-minutes"], 1);
  assert.equal(retry.if, "${{ !cancelled() && steps.standalone-download.outcome == 'failure' }}");
  assert.equal(retry["timeout-minutes"], 2);
  assert.equal(retry["continue-on-error"] ?? false, false);
  assert.equal(first["timeout-minutes"] + retry["timeout-minutes"], 3);
  assert.match(first.uses, /^actions\/download-artifact@[a-f0-9]{40}$/);
  assert.equal(retry.uses, first.uses);
  assert.deepEqual(retry.with, first.with);
  assert.equal(first.with["run-id"], "${{ needs.standalone-build.outputs.artifact-run-id }}");
  assert.equal(first.with.name, "standalone-library");
  assert.equal(first.with.path, "target/standalone-bundle");
  const execution = job.steps.find(
    (step) => step.name === "Test standalone Rust library without a CLI build",
  );
  assert.ok(job.steps.indexOf(execution) > job.steps.indexOf(retry));
  assert.equal(execution.if, undefined);
  assert.equal(job["continue-on-error"] ?? false, false);
});
