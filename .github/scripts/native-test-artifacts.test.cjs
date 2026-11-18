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
  batchPlan,
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
    const identities = (rows) => rows.map((row) => `${row.executable}:${row.name}`).sort();
    for (const count of [2, 6]) {
      const shards = Array.from({ length: count }, (_, index) =>
        casePlan(platform, entries, inventory, index + 1, count),
      ).flat();
      assert.deepEqual(identities(shards), identities(full));
      assert.equal(new Set(identities(shards)).size, full.length);
      const batched = Array.from({ length: count }, (_, index) =>
        batchPlan(casePlan(platform, entries, inventory, index + 1, count)),
      ).flat();
      assert.deepEqual(
        batched.flatMap((batch) => batch.names.map((name) => `${batch.executable}:${name}`)).sort(),
        identities(full),
      );
    }
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
        if (platform === "macOS") {
          for (const index of [1, 2]) {
            const weighted = invoke(["run", String(index), "2"]);
            assert.equal(weighted.status, 0, weighted.stderr);
            const report = JSON.parse(
              fs.readFileSync(path.join(directory, "target/native-rust-shard.json"), "utf8"),
            );
            assert.equal(report.scheduling.strategy, "macOS-two-shard-outlier-LPT");
            assert.equal(report.scheduling.defaultWeight, 1);
            assert.equal(report.scheduling.caseWeights.length, report.plan.length);
            assert.ok(
              report.scheduling.caseWeights.every((row) => row.source === "unmeasured-placeholder"),
            );
          }
          fs.rmSync(path.join(directory, "executions.jsonl"));
        }
        const results = [];
        for (let index = 1; index <= 6; index++) {
          const run = invoke(["run", String(index), "6"]);
          assert.equal(run.status, 0, run.stderr);
          const report = JSON.parse(
            fs.readFileSync(path.join(directory, "target/native-rust-shard.json"), "utf8"),
          );
          assert.equal(report.index, index);
          assert.deepEqual(
            report.batches.flatMap((batch) => batch.names).sort(),
            report.plan.map((entry) => entry.name).sort(),
          );
          assert.ok(
            report.batches.length < report.plan.length,
            "same-harness cases should share a process",
          );
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
          rows.flatMap((row) => row.tests).length,
          new Set(rows.flatMap((row) => row.tests.map((name) => `${row.executable}:${name}`))).size,
        );
        assert.deepEqual(results.flatMap((row) => row.names).sort(), selectedNames);
        assert.ok(
          results.some((row) => row.names.length > 1),
          "archive replay must batch testcase execution",
        );
        assert.ok(results.every((row) => row.exitCode === 0 && row.elapsedSeconds >= 0));
        if (platform === "Windows")
          assert.ok(
            rows
              .filter((row) => row.executable.includes("windows_job_object"))
              .every((row) => row.args.includes("--test-threads=1")),
          );
        assert.ok(
          rows.every(
            (row) =>
              fs.realpathSync(row.cwd) ===
              fs.realpathSync(path.join(directory, "crates/no-mistakes")),
          ),
        );
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
        assert.ok(failure.names.length > 0);
        const signalled = invoke(["run", "1", "6"], { NATIVE_TEST_FIXTURE_SIGNAL: "SIGTERM" });
        assert.equal(signalled.status, 1, signalled.stderr);
        const termination = JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-rust-results.jsonl"), "utf8").trim(),
        );
        assert.equal(termination.exitCode, null);
        assert.equal(termination.signal, "SIGTERM");
        assert.deepEqual(termination.names, failure.names);
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

test("native batches group only the same harness and original exact flags", () => {
  const plan = require("../../test-cases/native-test-artifacts/batches.json");
  const batches = batchPlan(plan);
  assert.equal(batches.length, 3);
  assert.deepEqual(batches[0].names, ["module::alpha", "module::beta"]);
  assert.deepEqual(batches[0].args, [
    "module::alpha",
    "module::beta",
    "--exact",
    "--test-threads=1",
  ]);
  assert.deepEqual(batches[2].args, ["module::gamma", "--exact", "--test-threads=2"]);
  assert.deepEqual(
    batches.flatMap((batch) => batch.names).sort(),
    plan.map((entry) => entry.name).sort(),
  );
  assert.throws(() => batchPlan([plan[0], plan[0]]), /more than once/);
  assert.throws(() => batchPlan([{ ...plan[0], args: [plan[0].name] }]), /exact selected/);
});

test("native batching splits before the conservative Windows argument budget", () => {
  const plan = require("../../test-cases/native-test-artifacts/batches.json");
  const selected = [plan[0], plan[2]].map((entry) => ({
    ...entry,
    name: entry.name.repeat(20),
    args: [entry.name.repeat(20), ...entry.args.slice(1)],
  }));
  const batches = batchPlan(selected, 1000);
  assert.equal(batches.length, 2);
  assert.deepEqual(
    batches.flatMap((batch) => batch.names),
    selected.map((entry) => entry.name),
  );
  for (const batch of batches) {
    const actualUnquotedLength = [path.resolve(batch.executable), ...batch.args].join(" ").length;
    assert.ok(actualUnquotedLength < 1000);
  }
  const huge = plan[0].name.repeat(2000);
  assert.throws(
    () => batchPlan([{ ...plan[0], name: huge, args: [huge, "--exact"] }]),
    /Windows argument budget/,
  );
  assert.throws(() => batchPlan(plan, 0), /argument budget/);
});
