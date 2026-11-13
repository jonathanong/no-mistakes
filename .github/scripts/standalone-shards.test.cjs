const assert = require("node:assert/strict");
const path = require("node:path");
const { test } = require("node:test");
const { commands } = require("./standalone-shards.cjs");

test("standalone build retains the default-feature library-only scope", () => {
  const [command] = commands("build", "target/bundle");
  assert.deepEqual(command.slice(0, 7), [
    "nextest",
    "archive",
    "--locked",
    "-p",
    "no-mistakes",
    "--lib",
    "--archive-file",
  ]);
  assert.equal(command[7], path.resolve("target/bundle/standalone.tar.zst"));
  assert.ok(!command.includes("--all-features"));
  assert.ok(!command.includes("--bins"));
});

test("every standalone partition replays its archive without building", () => {
  for (let index = 1; index <= 4; index += 1) {
    const [command] = commands("run", "target/bundle", index, 4, "/workspace");
    assert.deepEqual(command, [
      "nextest",
      "run",
      "--archive-file",
      path.resolve("target/bundle/standalone.tar.zst"),
      "--workspace-remap",
      "/workspace",
      "--extract-to",
      path.resolve("target/standalone-lib"),
      "--partition",
      `hash:${index}/4`,
      "--no-fail-fast",
      "--retries",
      "0",
    ]);
  }
});

test("invalid standalone partitions fail before invoking a compiler", () => {
  for (const [index, total] of [
    [0, 4],
    [5, 4],
    [1.5, 4],
    [1, 0],
    [1, NaN],
  ])
    assert.throws(() => commands("run", "bundle", index, total), /one-based/);
});
