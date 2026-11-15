const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
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
  assert.doesNotMatch(job, /--accept\b/);
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
