const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const test = globalThis.test || require("node:test").test;

const runtime = path.resolve(__dirname, "../../test-cases/rules/coverage-producer-assertion-owner-cleanup/runtime");
test("cleanup ownership fixture executes rows and reaches its configured assertions", {
  skip: Number(process.versions.node.split(".")[0]) < 22,
}, () => {
  const result = spawnSync(process.execPath, [
    "--experimental-strip-types", "--import", pathToFileURL(path.join(runtime, "register.mjs")).href,
    path.join(runtime, "run.mjs"),
  ], { encoding: "utf8", timeout: 10000 });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), { checked: 13 });
});
