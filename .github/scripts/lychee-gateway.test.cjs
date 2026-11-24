const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { classify, gatewayOnly } = require("./lychee-gateway.cjs");
const { parse } = require("yaml");
const { run } = require("./lychee-gateway-fixture.cjs");

const root = path.resolve(__dirname, "../..");
const saved = path.join(root, "test-cases/ci/lychee-gateway");
const read = (name) => readFileSync(path.join(saved, name), "utf8");

test("only an all-gateway lychee report is retried", () => {
  assert.deepEqual(classify(read("gateway.md")), { errors: 1, codes: ["502"] });
  assert.equal(gatewayOnly(read("gateway.md")), true);
  assert.equal(gatewayOnly(read("not-found.md")), false);
  assert.equal(gatewayOnly(read("mixed.md")), false);
  assert.equal(gatewayOnly(read("unparsed.md")), false);
  assert.equal(gatewayOnly(read("clean.md")), false);
});

test("documentation links retry the saved checker only for gateway failures", () => {
  const workflow = parse(readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8"));
  const job = workflow.jobs["quality-links"];
  const checkout = job.steps.find((step) => step.uses?.startsWith("actions/checkout@"));
  const depth = checkout.with?.["fetch-depth"];
  assert.ok(depth === 0 || depth >= 2, "PR link scope needs the merge checkout's first parent");
  const initial = job.steps.find((step) => step.id === "links");
  const retry = job.steps.find((step) => step.run?.includes("lychee-gateway.cjs"));
  assert.equal(initial["continue-on-error"], true);
  assert.equal(initial.if, "${{ steps.link-inputs.outputs.has-inputs == 'true' }}");
  assert.equal(initial.with.failIfEmpty, "${{ github.event_name != 'pull_request' }}");
  assert.equal(initial.with.args, "${{ env.LYCHEE_ARGS }}");
  assert.equal(retry.if, "${{ steps.links.outcome == 'failure' }}");
  assert.equal(initial.with.token, "${{ github.token }}");
  assert.equal(retry.env.GITHUB_TOKEN, initial.with.token);
  assert.notEqual(retry["continue-on-error"], true);
  assert.equal(job.steps.filter((step) => step["continue-on-error"] === true).length, 1);
  assert.equal(job.env.LYCHEE_GATEWAY_WAIT, "10,20");
  const preserve = job.steps.find(
    (step) => step.name === "Preserve final documentation link report",
  );
  assert.equal(preserve.if, "${{ !cancelled() && steps.links.outcome == 'failure' }}");
  assert.match(preserve.run, /GITHUB_STEP_SUMMARY/);
  assert.match(job.env.LYCHEE_ARGS, /--files-from "\$RUNNER_TEMP\/lychee-inputs\.txt"/);
  assert.match(
    job.env.LYCHEE_ARGS,
    /--exclude-path 'docs\/performance\/2238\/39488578982\/\(README\|tables\)\\.md\$'/,
  );
  assert.doesNotMatch(job.env.LYCHEE_ARGS, /--accept\b/);
  assert.ok(workflow.jobs.quality.needs.includes("quality-links"));
  assert.ok(workflow.jobs["tests-gate"].needs.includes("quality"));
  const guide = readFileSync(path.join(root, "docs/README.md"), "utf8");
  assert.match(
    guide,
    /--exclude-path 'docs\/performance\/2238\/39488578982\/\(README\|tables\)\\.md\$'/,
  );
});

test("hash-locked arm notes keep their recorded actions url", () => {
  const dir = path.join(root, "docs/performance/2238/39488578982");
  const checksums = JSON.parse(readFileSync(path.join(dir, "checksums.json"), "utf8"));
  for (const name of ["README.md", "tables.md"]) {
    const bytes = readFileSync(path.join(dir, name));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), checksums[name]);
    assert.match(bytes.toString("utf8"), /actions\/runs\/39488578982/);
  }
});

test("narrative performance notes name expired actions runs without linking them", () => {
  const notes = [
    "docs/performance/2238/shepherd-follow-up.md",
    "docs/performance/receiver-native-hosted-evidence-2012/README.md",
  ];
  for (const note of notes) {
    const text = readFileSync(path.join(root, note), "utf8");
    assert.doesNotMatch(text, /actions\/runs\//);
  }
  const followUp = readFileSync(
    path.join(root, "docs/performance/2238/shepherd-follow-up.md"),
    "utf8",
  );
  assert.match(followUp, /39495941623/);
  assert.match(followUp, /39511999832/);
  const receiver = readFileSync(
    path.join(root, "docs/performance/receiver-native-hosted-evidence-2012/README.md"),
    "utf8",
  );
  assert.match(receiver, /38654992458/);
});

test("a timeout in a selected document still fails the link check", () => {
  const result = run("timeout-only.md", { LYCHEE_ARGS: "--no-progress docs/unchanged.md" });
  assert.equal(result.status, 2);
  assert.equal(result.count, "");
});

test("a rejected page is not checked again", () => {
  const result = run("not-found.md", { LYCHEE_ARGS: "--no-progress README.md" });
  assert.equal(result.status, 2);
  assert.equal(result.count, "");
});

test(
  "a gateway failure is checked again and can pass",
  { skip: process.platform === "win32" },
  () => {
    const result = run("gateway.md", {
      LYCHEE_ARGS: "--no-progress README.md",
      LYCHEE_FAKE_SUCCEED_ON: "1",
    });
    assert.equal(result.status, 0);
    assert.equal(result.count, "1");
    assert.match(result.argv, /--output \S+ --no-progress README\.md/);
    assert.match(result.report, /Errors\s+\|\s+0/);
  },
);

test("a later non-gateway result stops the retry", () => {
  const result = run("gateway.md", {
    LYCHEE_ARGS: "--no-progress README.md",
    LYCHEE_FAKE_SUCCEED_ON: "9",
    LYCHEE_FAKE_FAILURE: path.join(saved, "not-found.md"),
  });
  assert.equal(result.status, 2);
  assert.equal(result.count, "1");
});

test(
  "two gateway responses can still clear on the last attempt",
  { skip: process.platform === "win32" },
  () => {
    const result = run("gateway.md", {
      LYCHEE_ARGS: "--no-progress README.md",
      LYCHEE_GATEWAY_WAIT: "0,0",
      LYCHEE_FAKE_SUCCEED_ON: "2",
    });
    assert.equal(result.status, 0);
    assert.equal(result.count, "2");
  },
);

test(
  "a gateway report without checker arguments is not retried",
  { skip: process.platform === "win32" },
  () => {
    const result = run("gateway.md", { LYCHEE_ARGS: "" });
    assert.equal(result.status, 1);
    assert.equal(result.count, "");
  },
);

test(
  "a nested glob is not expanded by bash before lychee",
  { skip: process.platform === "win32" },
  () => {
    const pattern = path.join(saved, "nested/docs/**/*.md");
    const result = run("gateway.md", {
      LYCHEE_ARGS: `--no-progress ${pattern}`,
      LYCHEE_FAKE_SUCCEED_ON: "1",
    });
    assert.equal(result.status, 0);
    assert.equal(result.count, "1");
    assert.match(result.argv, /nested\/docs\/\*\*\/\*\.md/);
    assert.doesNotMatch(result.argv, /(?:top|mid|deep)\.md/);
  },
);
