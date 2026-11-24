const assert = require("node:assert/strict");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");

const workflow = readWorkflow(path.join(__dirname, "../workflows/benchmarks.yml"));
const build = workflow.jobs.bench.steps;
const install = build.find((step) => step.name === "Install cargo-codspeed");

test("the cargo-codspeed pin matches the version reviewed for the benchmark rustflags", () => {
  assert.match(
    install.run,
    /^cargo install cargo-codspeed --version 5\.0\.1 --locked\s*$/,
    "Re-derive the encoded rustflags from the new cargo-codspeed version before updating this pin.",
  );
});

for (const name of ["Build CPU benchmarks", "Build memory benchmarks"]) {
  test(`${name} gets the reviewed rustflags`, () => {
    const step = build.find((candidate) => candidate.name === name);
    assert.deepEqual(step.env?.CARGO_ENCODED_RUSTFLAGS?.split("\x1f"), [
      ...workflow.env.RUSTFLAGS.split(" "),
      "-Cdebuginfo=line-tables-only",
      "-Cstrip=none",
      "--cfg=codspeed",
      "--cfg=codspeed_build",
    ]);
  });
}
