const { spawnSync } = require("node:child_process");
const { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const saved = path.resolve(__dirname, "../../test-cases/ci/lychee-gateway");

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
  const result = spawnSync(process.execPath, [path.join(__dirname, "lychee-gateway.cjs")], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${bin}:${process.env.PATH}`,
      LYCHEE_REPORT: report,
      LYCHEE_GATEWAY_WAIT: "0",
      LYCHEE_FAKE_COUNT: path.join(directory, "count"),
      LYCHEE_FAKE_ARGV: path.join(directory, "argv"),
      LYCHEE_FAKE_SUCCESS: path.join(saved, "clean.md"),
      LYCHEE_FAKE_FAILURE: path.join(saved, "gateway.md"),
      LYCHEE_FAKE_SUCCEED_ON: "9",
      LYCHEE_FAKE_EXIT: "2",
      LYCHEE_FAKE_NO_REPORT: "0",
      ...env,
    },
  });
  try {
    return {
      status: result.status,
      count: optionalRead(path.join(directory, "count")).trim(),
      argv: optionalRead(path.join(directory, "argv")),
      report: optionalRead(report),
      output: result.stdout + result.stderr,
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

module.exports = { run };
