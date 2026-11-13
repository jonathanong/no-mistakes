const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { Readable } = require("node:stream");
const { test } = require("node:test");
const {
  collectArtifacts,
  relativeExecutable,
  executionPlan,
  parseListing,
  casePlan,
} = require("./native-test-artifacts.cjs");
const { targets } = require("./native-rust-tests.cjs");
const expected = require("../../test-cases/native-rust-tests/expected.json");
const saved = path.resolve(__dirname, "../../test-cases/native-test-artifacts");
const wrapper = path.join(__dirname, "native-test-archive.cjs");

test("Cargo JSON artifacts retain test and ordinary CLI executables", async () => {
  const input = fs.readFileSync(path.join(saved, "messages.jsonl"));
  const stream = Readable.from([input.subarray(0, 19), input.subarray(19)]);
  const artifacts = collectArtifacts(stream);
  await new Promise((resolve) => stream.on("end", resolve));
  assert.equal(artifacts.length, 2);
  assert.equal(artifacts[0].test, true);
  assert.equal(artifacts[1].test, false);
});

for (const platform of ["macOS", "Windows"]) {
  test(`${platform} shards preserve the entire existing native filter inventory once`, () => {
    const entries = targets(platform).map(({ selector }) => ({
      name: selector[1],
      kind: selector[0].slice(2),
      executable: selector[1],
    }));
    const all = executionPlan(platform, entries);
    assert.deepEqual(
      all,
      expected[platform].map((args) => ({
        executable: args[1],
        args: args.slice(2).filter((arg) => arg !== "--"),
      })),
    );
    const cases = require("../../test-cases/native-test-artifacts/fixture/cases.json");
    const inventory = all.map((group) => ({
      ...group,
      tests: cases[group.executable].filter(
        (name) => !group.args[0] || group.args[0].startsWith("--") || name.includes(group.args[0]),
      ),
    }));
    const full = casePlan(platform, entries, inventory, 1, 1);
    const shards = Array.from({ length: 6 }, (_, index) =>
      casePlan(platform, entries, inventory, index + 1, 6),
    ).flat();
    const identities = (rows) => rows.map((row) => `${row.executable}:${row.name}`).sort();
    assert.deepEqual(identities(shards), identities(full));
    assert.equal(new Set(identities(shards)).size, full.length);
    assert.ok(full.every((entry) => !entry.name.startsWith("unselected")));
    assert.ok(
      full.every((entry) => entry.args[0] === entry.name && entry.args.includes("--exact")),
    );
    if (platform === "Windows")
      assert.ok(
        full
          .filter((entry) => entry.executable === "windows_job_object")
          .every((entry) => entry.args.includes("--test-threads=1")),
      );
    assert.throws(() => casePlan(platform, entries, inventory, 0, 6), /shard/);
    assert.throws(() => casePlan(platform, entries, [], 1, 6), /incomplete/);
    assert.throws(
      () =>
        casePlan(
          platform,
          entries,
          inventory.map((group) => ({ ...group, tests: [] })),
          1,
          6,
        ),
      /no tests/,
    );
    assert.throws(
      () =>
        casePlan(
          platform,
          entries,
          inventory.map((group) => ({ ...group, tests: [group.tests[0], group.tests[0]] })),
          1,
          6,
        ),
      /more than once/,
    );
    assert.throws(
      () =>
        casePlan(
          platform,
          entries,
          inventory.map((group) => ({ ...group, tests: [null] })),
          1,
          6,
        ),
      /malformed/,
    );
    assert.throws(
      () =>
        casePlan(
          platform,
          entries,
          inventory.map((group) =>
            group.args.some((arg) => !arg.startsWith("--"))
              ? { ...group, tests: ["unselected"] }
              : group,
          ),
          1,
          6,
        ),
      /outside its original filter/,
    );
    assert.throws(
      () =>
        casePlan(
          platform,
          entries,
          inventory.map((group) => ({ ...group, args: [] })),
          1,
          6,
        ),
      /differs from the original filters/,
    );
    assert.throws(() => executionPlan(platform, entries.slice(1)), /incomplete/);
    assert.throws(() => executionPlan(platform, [...entries].reverse()), /target mismatch/);
  });
}

test("archive paths cannot escape debug outputs", () => {
  const root = path.resolve("/repo");
  assert.equal(relativeExecutable(root, path.join(root, "target/debug/test")), "target/debug/test");
  assert.throws(() => relativeExecutable(root, path.resolve("/outside/test")), /outside/);
  assert.throws(() => relativeExecutable(root, path.join(root, "target/release/test")), /debug/);
});

for (const platform of ["macOS", "Windows"]) {
  test(
    `native ${platform} archive builds once and replays all filters without Cargo`,
    { skip: process.platform === "win32" },
    () => {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-archive-test-"));
      fs.cpSync(path.join(saved, "fixture"), directory, { recursive: true });
      fs.chmodSync(path.join(directory, "bin/cargo"), 0o755);
      const options = {
        cwd: directory,
        encoding: "utf8",
        timeout: 30000,
        env: {
          ...process.env,
          RUNNER_OS: platform,
          NO_MISTAKES_BUILD_NAPI: "1",
          GITHUB_SHA: "fixture-sha",
          NATIVE_FIXTURE_EXECUTIONS: path.join(directory, "executions.jsonl"),
          NATIVE_FIXTURE_CASES: path.join(directory, "cases.json"),
          PATH: `${path.join(directory, "bin")}${path.delimiter}${process.env.PATH}`,
        },
      };
      const invoke = (args, env = {}) =>
        spawnSync(process.execPath, [wrapper, ...args], {
          ...options,
          env: { ...options.env, ...env },
        });
      try {
        const disabled = invoke(["build"], { NO_MISTAKES_BUILD_NAPI: "0" });
        assert.equal(disabled.status, 1);
        assert.match(disabled.stderr, /require NO_MISTAKES_BUILD_NAPI=1/);
        const failedBuild = invoke(["build"], { NATIVE_TEST_FIXTURE_EXIT: "17" });
        assert.equal(failedBuild.status, 17, failedBuild.stderr);
        assert.equal(
          fs.existsSync(path.join(directory, "target/native-test-artifacts/manifest.json")),
          false,
        );
        const malformed = invoke(["build"], {
          NATIVE_FIXTURE_LISTING: path.join(saved, "listings/malformed.txt"),
        });
        assert.equal(malformed.status, 1, malformed.stderr);
        assert.match(malformed.stderr, /malformed native test listing/);
        assert.equal(
          fs.existsSync(path.join(directory, "target/native-test-artifacts/manifest.json")),
          false,
        );
        const built = invoke(["build"]);
        assert.equal(built.status, 0, built.stderr);
        const manifest = JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-test-artifacts/manifest.json")),
        );
        assert.equal(manifest.entries.length, targets(platform).length);
        fs.rmSync(path.join(directory, "target/debug"), { recursive: true });
        fs.rmSync(path.join(directory, "bin/cargo"));
        const results = [];
        for (let index = 1; index <= 6; index++) {
          const run = invoke(["run", String(index), "6"]);
          assert.equal(run.status, 0, run.stderr);
          const report = JSON.parse(
            fs.readFileSync(path.join(directory, "target/native-rust-shard.json"), "utf8"),
          );
          assert.equal(report.index, index);
          results.push(
            ...fs
              .readFileSync(path.join(directory, "target/native-rust-results.jsonl"), "utf8")
              .trim()
              .split("\n")
              .filter(Boolean)
              .map(JSON.parse),
          );
        }
        const rows = fs
          .readFileSync(path.join(directory, "executions.jsonl"), "utf8")
          .trim()
          .split("\n")
          .map(JSON.parse);
        const inventory = JSON.parse(
          fs.readFileSync(
            path.join(directory, "target/native-test-artifacts/inventory.json"),
            "utf8",
          ),
        );
        const selectedNames = inventory.flatMap((group) => group.tests).sort();
        assert.deepEqual(rows.flatMap((row) => row.tests).sort(), selectedNames);
        assert.equal(
          rows.length,
          new Set(rows.map((row) => `${row.executable}:${row.tests[0]}`)).size,
        );
        assert.deepEqual(results.map((row) => row.name).sort(), selectedNames);
        assert.ok(results.every((row) => row.exitCode === 0 && row.elapsedSeconds >= 0));
        if (platform === "Windows")
          assert.ok(
            rows
              .filter((row) => row.executable.includes("windows_job_object"))
              .every((row) => row.args.includes("--test-threads=1")),
          );
        assert.ok(rows.every((row) => row.cwd === path.join(directory, "crates/no-mistakes")));
        assert.equal(
          fs.readFileSync(path.join(directory, "compile.jsonl"), "utf8").trim().split("\n").length,
          targets(platform).length * 2 + 1,
        );
        const failed = invoke(["run", "1", "6"], { NATIVE_TEST_FIXTURE_EXIT: "17" });
        assert.equal(failed.status, 17, failed.stderr);
        const failure = JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-rust-results.jsonl"), "utf8").trim(),
        );
        assert.equal(failure.exitCode, 17);
        const wrongSource = invoke(["run", "1", "6"], { GITHUB_SHA: "different" });
        assert.equal(wrongSource.status, 1);
        assert.match(wrongSource.stderr, /source SHA mismatch/);
        const inventoryPath = path.join(directory, "target/native-test-artifacts/inventory.json");
        const backup = path.join(directory, "inventory-backup.json");
        fs.copyFileSync(inventoryPath, backup);
        fs.copyFileSync(path.join(saved, "messages.jsonl"), inventoryPath);
        const corruptInventory = invoke(["run", "1", "6"]);
        assert.equal(corruptInventory.status, 1);
        assert.match(corruptInventory.stderr, /inventory checksum mismatch/);
        fs.copyFileSync(backup, inventoryPath);
        const file = manifest.files[0].path;
        fs.copyFileSync(
          path.join(saved, "messages.jsonl"),
          path.join(directory, "target/native-test-artifacts", file),
        );
        const corrupt = invoke(["run", "1", "6"]);
        assert.equal(corrupt.status, 1);
        assert.match(corrupt.stderr, /checksum mismatch/);
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  );
}

test("libtest listings reject missing, malformed, duplicate and miscounted cases", () => {
  const listing = (name) => fs.readFileSync(path.join(saved, "listings", `${name}.txt`), "utf8");
  assert.deepEqual(parseListing(listing("valid")), [
    "module::alpha",
    "module::beta",
    "module::gamma",
  ]);
  assert.deepEqual(
    parseListing(listing("valid").replaceAll("\n", "\r\n")),
    parseListing(listing("valid")),
  );
  for (const name of ["empty", "malformed", "duplicate", "wrong-count"])
    assert.throws(() => parseListing(listing(name)), /native test/);
});
