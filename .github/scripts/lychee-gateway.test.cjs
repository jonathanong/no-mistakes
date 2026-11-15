const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { mkdirSync, mkdtempSync, readFileSync, symlinkSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { classify, gatewayOnly, main } = require("./lychee-gateway.cjs");
const { workflowJob } = require("./workflow-contract.cjs");

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
  const job = workflowJob(readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8"), "quality-links");
  assert.match(job, /continue-on-error: true/);
  assert.match(job, /steps\.links\.outcome == 'failure'/);
  assert.match(job, /node \.github\/scripts\/lychee-gateway\.cjs/);
  assert.match(job, /args: \$\{\{ env\.LYCHEE_ARGS \}\}/);
  assert.match(job, /LYCHEE_ARGS:/);
  assert.match(job, /'docs\/\*\*\/\*\.md'/);
  assert.match(job, /'plugins\/\*\/skills\/\*\*\/\*\.md'/);
  assert.match(job, /--exclude-path 'docs\/performance\/2238\/39488578982\/\(README\|tables\)\\.md\$'/);
  assert.doesNotMatch(job, /--accept\b/);
  const guide = readFileSync(path.join(root, "docs/README.md"), "utf8");
  assert.match(guide, /--exclude-path 'docs\/performance\/2238\/39488578982\/\(README\|tables\)\\.md\$'/);
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
  const followUp = readFileSync(path.join(root, "docs/performance/2238/shepherd-follow-up.md"), "utf8");
  assert.match(followUp, /39495941623/);
  assert.match(followUp, /39511999832/);
  const receiver = readFileSync(
    path.join(root, "docs/performance/receiver-native-hosted-evidence-2012/README.md"),
    "utf8",
  );
  assert.match(receiver, /38654992458/);
});

function optionalRead(file) {
  try {
    return readFileSync(file, "utf8");
  } catch (error) {
    if (error.code === "ENOENT") return "";
    throw error;
  }
}

function run(reportName, env) {
  const directory = mkdtempSync(path.join(tmpdir(), "lychee-gateway-"));
  const bin = path.join(directory, "bin");
  mkdirSync(bin);
  symlinkSync(path.join(saved, "fake-lychee.sh"), path.join(bin, "lychee"));
  const report = path.join(directory, "lychee", "out.md");
  mkdirSync(path.dirname(report));
  spawnSync("cp", [path.join(saved, reportName), report]);
  const previous = {
    PATH: process.env.PATH,
    LYCHEE_REPORT: process.env.LYCHEE_REPORT,
    LYCHEE_ARGS: process.env.LYCHEE_ARGS,
    LYCHEE_GATEWAY_WAIT: process.env.LYCHEE_GATEWAY_WAIT,
    LYCHEE_FAKE_COUNT: process.env.LYCHEE_FAKE_COUNT,
    LYCHEE_FAKE_ARGV: process.env.LYCHEE_FAKE_ARGV,
    LYCHEE_FAKE_SUCCEED_ON: process.env.LYCHEE_FAKE_SUCCEED_ON,
    LYCHEE_FAKE_SUCCESS: process.env.LYCHEE_FAKE_SUCCESS,
    LYCHEE_FAKE_FAILURE: process.env.LYCHEE_FAKE_FAILURE,
  };
  Object.assign(process.env, {
    PATH: `${bin}:${process.env.PATH}`,
    LYCHEE_REPORT: report,
    LYCHEE_GATEWAY_WAIT: "0",
    LYCHEE_FAKE_COUNT: path.join(directory, "count"),
    LYCHEE_FAKE_ARGV: path.join(directory, "argv"),
    LYCHEE_FAKE_SUCCESS: path.join(saved, "clean.md"),
    LYCHEE_FAKE_FAILURE: path.join(saved, "gateway.md"),
    ...env,
  });
  try {
    return {
      status: main(),
      count: optionalRead(path.join(directory, "count")).trim(),
      argv: optionalRead(path.join(directory, "argv")),
      report: readFileSync(report, "utf8"),
    };
  } finally {
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
}

test("a rejected page is not checked again", { skip: process.platform === "win32" }, () => {
  const result = run("not-found.md", { LYCHEE_ARGS: "--no-progress README.md" });
  assert.equal(result.status, 2);
  assert.equal(result.count, "");
});

test("a gateway failure is checked again and can pass", { skip: process.platform === "win32" }, () => {
  const result = run("gateway.md", {
    LYCHEE_ARGS: "--no-progress README.md",
    LYCHEE_FAKE_SUCCEED_ON: "1",
  });
  assert.equal(result.status, 0);
  assert.equal(result.count, "1");
  assert.match(result.argv, /--output \S+ --no-progress README\.md/);
  assert.match(result.report, /Errors\s+\|\s+0/);
});

test("a later non-gateway result stops the retry", { skip: process.platform === "win32" }, () => {
  const result = run("gateway.md", {
    LYCHEE_ARGS: "--no-progress README.md",
    LYCHEE_FAKE_SUCCEED_ON: "9",
    LYCHEE_FAKE_FAILURE: path.join(saved, "not-found.md"),
  });
  assert.equal(result.status, 2);
  assert.equal(result.count, "1");
});

test("two gateway responses can still clear on the last attempt", { skip: process.platform === "win32" }, () => {
  const result = run("gateway.md", {
    LYCHEE_ARGS: "--no-progress README.md",
    LYCHEE_GATEWAY_WAIT: "0,0",
    LYCHEE_FAKE_SUCCEED_ON: "2",
  });
  assert.equal(result.status, 0);
  assert.equal(result.count, "2");
});

test("a gateway report without checker arguments is not retried", { skip: process.platform === "win32" }, () => {
  const result = run("gateway.md", { LYCHEE_ARGS: "" });
  assert.equal(result.status, 1);
  assert.equal(result.count, "");
});

test("a nested glob is not expanded by bash before lychee", { skip: process.platform === "win32" }, () => {
  const pattern = path.join(saved, "nested/docs/**/*.md");
  const result = run("gateway.md", {
    LYCHEE_ARGS: `--no-progress ${pattern}`,
    LYCHEE_FAKE_SUCCEED_ON: "1",
  });
  assert.equal(result.status, 0);
  assert.equal(result.count, "1");
  assert.match(result.argv, /nested\/docs\/\*\*\/\*\.md/);
  assert.doesNotMatch(result.argv, /(?:top|mid|deep)\.md/);
});
