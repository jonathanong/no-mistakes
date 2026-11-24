const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { selectPreviousRunBase } = require("./push-diff-base.cjs");
const { readWorkflow } = require("./workflow-contract.cjs");

const root = path.resolve(__dirname, "../..");
const script = path.join(__dirname, "push-diff-base.cjs");
const fixturesDir = path.join(__dirname, "fixtures", "push-diff-base");

function readFixture(name) {
  return JSON.parse(readFileSync(path.join(fixturesDir, name), "utf8"));
}

function runCli(args, cwd) {
  return spawnSync(process.execPath, [script, ...args], { encoding: "utf8", cwd });
}

function git(repo, args) {
  const result = spawnSync(
    "git",
    [
      "-C",
      repo,
      "-c",
      "commit.gpgsign=false",
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=test",
      ...args,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, `${args.join(" ")}\n${result.stderr}`);
  return result.stdout.trim();
}

test("selects the most recent completed push run from saved API fixtures", () => {
  assert.equal(
    selectPreviousRunBase(readFixture("previous-completed.json"), "300"),
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  );
  assert.equal(
    selectPreviousRunBase(readFixture("previous-skipped.json"), "400"),
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  );
  assert.equal(
    selectPreviousRunBase(readFixture("previous-failed.json"), "300"),
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  );
  assert.equal(selectPreviousRunBase(readFixture("no-previous-run.json"), "300"), "");

  const cliSkipped = runCli([
    "--select-runs",
    path.join(fixturesDir, "previous-skipped.json"),
    "--current-run-id",
    "400",
  ]);
  assert.equal(cliSkipped.status, 0, cliSkipped.stderr);
  assert.equal(cliSkipped.stdout.trim(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
});

test("resolves skipped-commit base on shallow clone and falls back on non-ancestor or missing base", () => {
  const work = mkdtempSync(path.join(tmpdir(), "push-diff-base-"));
  try {
    const remote = path.join(work, "remote.git");
    const seed = path.join(work, "seed");
    const clone = path.join(work, "clone");
    const runsFile = path.join(work, "runs.json");

    git(work, ["init", "--bare", "-q", remote]);
    git(work, ["init", "-q", "-b", "main", seed]);
    git(seed, ["remote", "add", "origin", remote]);

    writeFileSync(path.join(seed, "a.txt"), "a\n");
    git(seed, ["add", "a.txt"]);
    git(seed, ["commit", "-qm", "commit A"]);
    const commitA = git(seed, ["rev-parse", "HEAD"]);

    writeFileSync(path.join(seed, "b.txt"), "b\n");
    git(seed, ["add", "b.txt"]);
    git(seed, ["commit", "-qm", "commit B"]);
    const commitB = git(seed, ["rev-parse", "HEAD"]);

    writeFileSync(path.join(seed, "c.txt"), "c\n");
    git(seed, ["add", "c.txt"]);
    git(seed, ["commit", "-qm", "commit C"]);
    const commitC = git(seed, ["rev-parse", "HEAD"]);
    git(seed, ["push", "-q", "origin", "main"]);

    // Unrelated commit simulating a force-push / rewritten history.
    git(seed, ["checkout", "-q", "--orphan", "rewritten"]);
    writeFileSync(path.join(seed, "x.txt"), "x\n");
    git(seed, ["add", "x.txt"]);
    git(seed, ["commit", "-qm", "commit X"]);
    const commitX = git(seed, ["rev-parse", "HEAD"]);
    git(seed, ["push", "-q", "origin", "rewritten"]);

    // Shallow clone matching actions/checkout fetch-depth: 2 (only C and B are local).
    git(work, ["clone", "-q", "--no-local", "--depth=2", "--branch", "main", remote, clone]);

    // 1. B was skipped; last completed run was A (outside depth=2). Must fetch A, deepen, and return A.
    const skippedPayload = readFixture("previous-skipped.json");
    skippedPayload.workflow_runs[0].head_sha = commitC;
    skippedPayload.workflow_runs[1].head_sha = commitB;
    skippedPayload.workflow_runs[3].head_sha = commitA;
    writeFileSync(runsFile, JSON.stringify(skippedPayload));

    const resolved = runCli(
      [
        "--event",
        "push",
        "--head",
        commitC,
        "--current-run-id",
        "400",
        "--runs-file",
        runsFile,
        "--cwd",
        clone,
      ],
      work,
    );
    assert.equal(resolved.status, 0, resolved.stderr);
    assert.equal(resolved.stdout.trim(), commitA);

    // 2. Non-ancestor base (force push): must fall back to empty (full scope).
    const nonAncestorPayload = readFixture("previous-completed.json");
    nonAncestorPayload.workflow_runs[1].head_sha = commitX;
    writeFileSync(runsFile, JSON.stringify(nonAncestorPayload));

    const nonAncestor = runCli(
      [
        "--event",
        "push",
        "--head",
        commitC,
        "--current-run-id",
        "300",
        "--runs-file",
        runsFile,
        "--cwd",
        clone,
      ],
      work,
    );
    assert.equal(nonAncestor.status, 0);
    assert.equal(nonAncestor.stdout.trim(), "");
    assert.match(nonAncestor.stderr, /not an ancestor/);

    // 3. Unfetchable base SHA: must fall back to empty (full scope).
    const missingPayload = readFixture("previous-completed.json");
    missingPayload.workflow_runs[1].head_sha = "1234567890abcdef1234567890abcdef12345678";
    writeFileSync(runsFile, JSON.stringify(missingPayload));

    const missing = runCli(
      [
        "--event",
        "push",
        "--head",
        commitC,
        "--current-run-id",
        "300",
        "--runs-file",
        runsFile,
        "--cwd",
        clone,
      ],
      work,
    );
    assert.equal(missing.status, 0);
    assert.equal(missing.stdout.trim(), "");
    assert.match(missing.stderr, /could not be fetched/);

    // 4. No previous completed run: must fall back to empty (full scope).
    const none = runCli(
      [
        "--event",
        "push",
        "--head",
        commitC,
        "--current-run-id",
        "300",
        "--runs-file",
        path.join(fixturesDir, "no-previous-run.json"),
        "--cwd",
        clone,
      ],
      work,
    );
    assert.equal(none.status, 0);
    assert.equal(none.stdout.trim(), "");
    assert.match(none.stderr, /no prior completed push run/);

    // 5. Pull request keeps its PR base unchanged without querying runs.
    const pr = runCli(["--event", "pull_request", "--head", commitC, "--pr-base", commitA], work);
    assert.equal(pr.status, 0, pr.stderr);
    assert.equal(pr.stdout.trim(), commitA);

    // 6. workflow_dispatch falls back to empty (full scope).
    const dispatch = runCli(["--event", "workflow_dispatch", "--head", commitC], work);
    assert.equal(dispatch.status, 0, dispatch.stderr);
    assert.equal(dispatch.stdout.trim(), "");
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test("ref-keyed workflows do not use github.event.before directly and wire push-diff-base.cjs", () => {
  const workflowsDir = path.join(root, ".github", "workflows");
  const files = readdirSync(workflowsDir).filter((f) => f.endsWith(".yml") || f.endsWith(".yaml"));
  let checkedRefKeyed = 0;

  for (const file of files) {
    const workflow = readWorkflow(path.join(workflowsDir, file));
    const triggers = workflow?.on;
    const hasPush =
      triggers === "push" ||
      (Array.isArray(triggers) && triggers.includes("push")) ||
      (triggers && typeof triggers === "object" && Object.hasOwn(triggers, "push"));
    const group = String(workflow?.concurrency?.group || "");
    if (!hasPush || !group.includes("github.ref") || group.includes("github.sha")) continue;

    for (const [jobName, job] of Object.entries(workflow.jobs || {})) {
      for (const step of job.steps || []) {
        const serialized = JSON.stringify({
          if: step.if,
          env: step.env,
          with: step.with,
          run: step.run,
        });
        assert.doesNotMatch(
          serialized,
          /github\.event\.before/,
          `${file} job ${jobName} step "${step.name}" must not use github.event.before directly`,
        );
      }
    }

    const changes = workflow.jobs?.changes;
    if (!changes) continue;
    checkedRefKeyed += 1;
    assert.equal(
      changes.permissions?.actions,
      "read",
      `${file} changes job must declare permissions.actions: read`,
    );
    const checkout = (changes.steps || []).find((s) =>
      String(s.uses || "").startsWith("actions/checkout@"),
    );
    assert.ok(checkout, `${file} changes job must check out the repository`);
    assert.notEqual(
      checkout.with?.["fetch-depth"],
      0,
      `${file} changes job must not use a full clone (fetch-depth: 0)`,
    );
    const resolver = (changes.steps || []).find((s) =>
      String(s.run || "").includes(".github/scripts/push-diff-base.cjs"),
    );
    assert.ok(resolver, `${file} changes job must call push-diff-base.cjs`);
    assert.match(
      String(resolver.run),
      new RegExp(`--workflow\\s+${file.replace(".", "\\.")}`),
      `${file} changes job must pass --workflow ${file}`,
    );
  }

  assert.ok(checkedRefKeyed >= 1, "expected at least one ref-keyed workflow to verify");

  const ci = readFileSync(path.join(workflowsDir, "ci.yml"), "utf8");
  assert.match(ci, /node --test[^\n]*push-diff-base\.test\.cjs/);
});
