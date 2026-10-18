const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { select, verify } = require("../../.github/scripts/hosted-check-reuse.cjs");

const root = join(__dirname, "../../fixtures/hosted-check/reuse");
const saved = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));
const source = "a".repeat(40);
const observed = { commit: source, tree: "b".repeat(40) };
const clone = () => structuredClone(saved);
const hash = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const check = (artifact = "artifact", checkout = "source", seen = observed, records = saved) =>
  verify("42", "current", source, join(root, artifact), join(root, checkout), seen, records);

test("reuse retains registered production bytes and original compiler provenance", () => {
  assert.equal(select("42", "current", source, saved).artifactId, 123);
  const before = hash(join(root, "artifact/provenance.json"));
  const result = check();
  assert.equal(result.reused, true);
  assert.equal(result.originalRunId, "42");
  assert.equal(result.originalArtifactId, 123);
  assert.equal(result.sourceTree, observed.tree);
  assert.equal(result.compiler, "synthetic compiler retained verbatim");
  assert.equal(hash(join(root, "artifact/provenance.json")), before);
});

test("reuse rejects unregistered runs, wrong roles, source identities and incomplete manifest", () => {
  for (const run of ["41", "0", "-42", "42;echo", "__proto__"])
    assert.throws(() => select(run, "current", source, saved), /registered/);
  assert.throws(() => select("42", "other", source, saved), /variant/);
  assert.throws(() => select("42", "current", "c".repeat(40), saved), /source/);
  const missing = clone();
  delete missing["42"].current;
  assert.throws(() => select("42", "current", source, missing), /source/);
  for (const id of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    const changed = clone();
    changed["42"].current.artifactId = id;
    assert.throws(() => select("42", "current", source, changed), /artifact ID/);
  }
  const partial = clone();
  delete partial["42"].current.files["cpu-probe"];
  assert.throws(() => select("42", "current", source, partial), /four production/);
  const malformed = clone();
  malformed["42"].current.files["cpu-probe"] = "bad";
  assert.throws(() => select("42", "current", source, malformed), /checksum/);
});

test("reuse verifies every immutable file and rejects extra, missing or non-file inputs", () => {
  for (const file of Object.keys(saved["42"].current.files)) {
    const changed = clone();
    changed["42"].current.files[file] = "0".repeat(64);
    assert.throws(() => check("artifact", "source", observed, changed), /hash mismatch/);
  }
  for (const artifact of ["artifact-extra", "artifact-missing"])
    assert.throws(() => check(artifact), /missing or extra/);
  assert.throws(() => check("artifact-directory"), /hash mismatch/);
  const changed = clone();
  changed["42"].current.files.SHA256SUMS = hash(join(root, "artifact-bad-checksum/SHA256SUMS"));
  assert.throws(
    () => check("artifact-bad-checksum", "source", observed, changed),
    /binary checksum/,
  );
});

test("reuse binds original Cargo inputs and the actual source checkout", () => {
  assert.throws(
    () => check("artifact", "source", { ...observed, commit: "c".repeat(40) }),
    /source identity/,
  );
  assert.throws(
    () => check("artifact", "source", { ...observed, tree: "c".repeat(40) }),
    /source identity/,
  );
  for (const checkout of ["source-wrong-lock", "source-wrong-manifest"])
    assert.throws(() => check("artifact", checkout), /Cargo inputs/);
  assert.throws(() => check("artifact", "source-no-rlib"), /crate type/);
});

test("workflow keeps reuse optional, skips production tools and scopes artifact reads to build", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  assert.match(workflow, /artifactRunId:/);
  assert.match(workflow, /expectedFindings:[\s\S]*?default: false/);
  const build = workflow.slice(workflow.indexOf("  build:"), workflow.indexOf("  sample:"));
  assert.match(build, /permissions:\n      contents: read\n      actions: read/);
  for (const name of [
    "Install production Rust compiler",
    "Cache production build",
    "Build with official x86 release flags and default features",
  ])
    assert.ok(
      build.includes(`- name: ${name}\n        if: \u0024{{ inputs.artifactRunId == '' }}`),
    );
  assert.match(build, /artifact-ids: \$\{\{ steps.reuse.outputs.artifact-id \}\}/);
  assert.match(build, /run-id: \$\{\{ inputs.artifactRunId \}\}/);
  assert.match(build, /github-token: \$\{\{ github.token \}\}/);
  assert.equal((workflow.match(/HOSTED_CHECK_EXPECTED_FINDINGS:/g) || []).length, 2);
});
