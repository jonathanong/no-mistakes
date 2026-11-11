const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const {
  appendFileSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} = require("node:fs");
const { tmpdir } = require("node:os");
const { dirname, join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const fixturePath = "fixtures/symbols/signature-impact-encoding/tolerated/binary.mts";
const binaryFixture = join(repoRoot, fixturePath);
const workflow = readFileSync(join(repoRoot, ".github/workflows/ci.yml"), "utf8");
const step = workflow
  .split("      - name: Check committed binary files\n")[1]
  .split("\n      - name:")[0];
const policy = step
  .split("        run: |\n")[1]
  .split("\n")
  .map((line) => line.replace(/^          /, ""))
  .join("\n");

function checkPolicy(prepare) {
  const directory = mkdtempSync(join(tmpdir(), "committed-binary-policy-"));
  const copy = (path, source = binaryFixture) => {
    const target = join(directory, path);
    mkdirSync(dirname(target), { recursive: true });
    copyFileSync(source, target);
    return target;
  };
  try {
    for (const args of [
      ["init", "--quiet"],
      ["config", "core.autocrlf", "false"],
    ]) {
      assert.equal(spawnSync("git", args, { cwd: directory }).status, 0);
    }
    prepare(copy);
    assert.equal(spawnSync("git", ["add", "--all"], { cwd: directory }).status, 0);
    return spawnSync("bash", ["-c", policy], { cwd: directory, encoding: "utf8" });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("binary gate accepts the exact saved source-decoding fixture", () => {
  const result = checkPolicy((copy) => copy(fixturePath));
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
});

test("binary gate rejects the same saved bytes at another path", () => {
  const result = checkPolicy((copy) => copy("unexpected.mts"));
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stdout, /file=unexpected.mts::Committed binary file detected/);
});

test("binary gate rejects changed binary content at the exception path", () => {
  const result = checkPolicy((copy) =>
    appendFileSync(copy(fixturePath), readFileSync(binaryFixture)),
  );
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stdout, /binary.mts::Committed binary file detected/);
});

test("approved fixture does not hide another committed binary", () => {
  const result = checkPolicy((copy) => {
    copy(fixturePath);
    copy("unexpected.mts");
  });
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stdout, /file=unexpected.mts::Committed binary file detected/);
  assert.doesNotMatch(result.stdout, /file=fixtures/);
});

test("binary gate continues accepting ordinary saved source text", () => {
  const result = checkPolicy((copy) =>
    copy("target.mts", join(repoRoot, dirname(fixturePath), "target.mts")),
  );
  assert.equal(result.status, 0, result.stderr);
});
