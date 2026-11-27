const assert = require("node:assert/strict");
const { isUtf8 } = require("node:buffer");
const { execFileSync } = require("node:child_process");
const { cpSync, lstatSync, mkdtempSync, readFileSync, renameSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { test } = require("node:test");

const root = join(__dirname, "../..");
const CONSUMER_TOKENS = ["voucha", "@data-stores/valkey", "voucha.ai"];

function visibleFiles(cwd) {
  const list = (...flags) =>
    execFileSync("git", ["ls-files", "-z", "--exclude-per-directory=.gitignore", ...flags], {
      cwd,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    })
      .split("\0")
      .filter(Boolean);
  const ignored = new Set(list("--cached", "--ignored"));
  return list("--cached", "--others").filter(
    (path) => !ignored.has(path) && lstatSync(join(cwd, path), { throwIfNoEntry: false })?.isFile(),
  );
}

test("the repository README documents test frameworks and links to the project guides", () => {
  const readme = readFileSync(join(root, "README.md"), "utf8");
  for (const text of ["<playwright\\|vitest>", "(docs/feature-parity.md)", "(docs/why.md)"])
    assert.ok(readme.includes(text), `README.md must contain ${text}`);
});

test("consumer identifiers occur only in the documented denylist", () => {
  const denylist = ".github/scripts/repository-doc-policy.test.cjs";
  const source = readFileSync(join(root, denylist), "utf8").toLowerCase();
  const declaration = `const consumer_tokens = [${CONSUMER_TOKENS.map((token) => JSON.stringify(token)).join(", ")}];`;
  assert.ok(source.includes(declaration), "keep the denylist exception in its token declaration");
  for (const [index, token] of CONSUMER_TOKENS.entries())
    assert.equal(
      source.split(token).length - 1,
      [2, 1, 1][index],
      `denylist occurrences of ${token}`,
    );

  const violations = [];
  for (const path of visibleFiles(root)) {
    if (path === denylist) continue;
    const bytes = readFileSync(join(root, path));
    if (!isUtf8(bytes)) continue;
    const lines = bytes.toString().toLowerCase().split("\n");
    for (const [index, line] of lines.entries()) {
      for (const token of CONSUMER_TOKENS) {
        if (line.includes(token))
          violations.push(`${path}:${index + 1} contains a forbidden consumer identifier`);
      }
    }
  }
  assert.ok(violations.length === 0, violations.sort().join("\n"));
});

test("the identifier scan includes root and agent docs and excludes ignored files and Git metadata", (t) => {
  const fixture = mkdtempSync(join(tmpdir(), "identifier-inventory-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  cpSync(join(root, "test-cases/docs/identifier-inventory/fixture"), fixture, { recursive: true });
  execFileSync("git", ["init", "-q"], { cwd: fixture });

  renameSync(join(fixture, ".git.fixture/config"), join(fixture, ".git/fixture"));
  assert.deepEqual(visibleFiles(fixture).sort(), [
    ".gitignore",
    "README.md",
    "skills/no-mistakes/SKILL.md",
    "skills/no-mistakes/references/example.md",
  ]);
});
