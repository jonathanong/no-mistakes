const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const { createWorkflowTopologyIndex } = require("../workflow-topology-index");

// Reuses the same golden `WorkflowTopology` JSON the Rust engine's
// fixture-backed tests assert byte-for-byte parity against (see
// `crates/no-mistakes/src/codebase/workflow_topology/tests.rs`), so this
// index is exercised against a real, non-trivial graph rather than a
// hand-rolled stub.
function fixtureTopology(name) {
  const path = join(
    __dirname,
    "..",
    "..",
    "..",
    "test-cases",
    "workflow-topology",
    name,
    "expected.json",
  );
  return JSON.parse(readFileSync(path, "utf8"));
}

test("job upstream/downstream traversal is direct and transitive", () => {
  const index = createWorkflowTopologyIndex(fixtureTopology("needs-basic"));
  const build = ".github/workflows/pipeline.yml#build";
  const test_ = ".github/workflows/pipeline.yml#test";
  const deploy = ".github/workflows/pipeline.yml#deploy";

  assert.deepEqual(index.directDownstreamJobIds(build), [test_]);
  assert.deepEqual(index.directUpstreamJobIds(deploy), [test_]);
  assert.deepEqual(index.transitiveDownstreamJobIds(build), [deploy, test_].sort());
  assert.deepEqual(index.transitiveUpstreamJobIds(deploy), [build, test_].sort());
  // Transitive traversal excludes the source itself.
  assert.equal(index.transitiveDownstreamJobIds(build).includes(build), false);
});

test("unknown job or workflow ids throw", () => {
  const index = createWorkflowTopologyIndex(fixtureTopology("needs-basic"));
  assert.throws(() => index.directUpstreamJobIds("nope"), /unknown workflow job: nope/);
  assert.throws(() => index.directCalleeWorkflowPaths("nope.yml"), /unknown workflow: nope\.yml/);
});

test("local reusable-workflow calls populate caller/callee traversals; remote calls do not", () => {
  const topology = fixtureTopology("reusable-calls");
  const index = createWorkflowTopologyIndex(topology);
  const caller = ".github/workflows/caller.yml";
  const callee = ".github/workflows/reusable-callee.yml";
  const goodJob = ".github/workflows/caller.yml#good";

  assert.deepEqual(index.directCallerWorkflowPaths(callee), [caller]);
  assert.deepEqual(index.directCallerJobIds(callee), [goodJob]);
  assert.deepEqual(index.directCalleeWorkflowPaths(caller).includes(callee), true);
  // cycle-a calls cycle-b calls cycle-a: transitively reachable, but the
  // source itself is excluded even though the cycle loops back to it.
  assert.deepEqual(index.transitiveCalleeWorkflowPaths(".github/workflows/cycle-a.yml"), [
    ".github/workflows/cycle-b.yml",
  ]);
});

test("workflow_run source/subscriber traversal and edge metadata", () => {
  const topology = fixtureTopology("workflow-run");
  const index = createWorkflowTopologyIndex(topology);
  const source = ".github/workflows/source.yml";
  const subscriberOk = ".github/workflows/subscriber-ok.yml";

  assert.deepEqual(index.directWorkflowRunSubscriberPaths(source), [subscriberOk]);
  assert.deepEqual(index.directWorkflowRunSourcePaths(subscriberOk), [source]);
  const outgoing = index.outgoingWorkflowRunEdges(source);
  assert.equal(outgoing.length, 1);
  assert.deepEqual(outgoing[0].types, ["completed"]);
  assert.deepEqual(outgoing[0].branches, ["main"]);
});

test("workflowsByPath/jobsById are frozen and read-only despite being backed by a real Map", () => {
  const index = createWorkflowTopologyIndex(fixtureTopology("needs-basic"));
  assert.equal(index.jobsById.size, 3);
  // `FrozenReadonlyMap` exposes no `.set`/`.delete` at all — not even a
  // no-op one — so calling either isn't just silently ignored, it's a
  // TypeError (not a function).
  assert.equal(typeof index.jobsById.set, "undefined");
  assert.equal(typeof index.jobsById.delete, "undefined");
  // The cloned job values are deep-frozen. A strict-mode assignment (ESM,
  // which vitest transforms into) throws TypeError; a sloppy-mode one
  // (plain CommonJS under `node --test`) silently no-ops — either way the
  // frozen value must not change, so tolerate both and assert on that.
  const job = index.jobsById.get(".github/workflows/pipeline.yml#build");
  const originalId = job.id;
  try {
    job.id = "mutated";
  } catch {
    // expected in strict-mode environments
  }
  assert.equal(job.id, originalId);
});

test("does not mutate or reference the source topology object", () => {
  const topology = fixtureTopology("needs-basic");
  const before = JSON.stringify(topology);
  createWorkflowTopologyIndex(topology);
  assert.equal(JSON.stringify(topology), before);
});
