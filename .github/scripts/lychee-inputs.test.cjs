const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { cpSync, mkdtempSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const script = path.join(__dirname, "lychee-inputs.cjs");
const fixture = path.resolve(__dirname, "../../test-cases/ci/lychee-inputs");

function selectedInputs(directory, event) {
  return execFileSync(process.execPath, [script], {
    cwd: directory,
    encoding: "utf8",
    env: { ...process.env, GITHUB_EVENT_NAME: event },
  })
    .split("\n")
    .filter(Boolean);
}

test("PR link sources come from the merge checkout's changes", (context) => {
  const directory = mkdtempSync(path.join(tmpdir(), "lychee-inputs-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  cpSync(fixture, directory, { recursive: true });
  const git = (...args) =>
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Test Author",
        "-c",
        "user.email=test@example.invalid",
        "-c",
        "commit.gpgSign=false",
        "-C",
        directory,
        ...args,
      ],
      { stdio: "pipe" },
    );

  git("init", "-b", "main");
  git("add", "docs/unchanged.md");
  git("commit", "-m", "Base documentation");
  git("switch", "-c", "feature");
  git("add", "src/change.txt");
  git("commit", "-m", "Change a source file");
  git("switch", "main");
  git("merge", "--no-ff", "feature");
  assert.deepEqual(selectedInputs(directory, "pull_request"), []);

  git("switch", "feature");
  git("add", "docs/changed [draft].md");
  git("commit", "-m", "Add a document");
  git("switch", "main");
  git("merge", "--no-ff", "feature");
  assert.deepEqual(selectedInputs(directory, "pull_request"), ["docs/changed [[]draft[]].md"]);

  git("switch", "feature");
  git("rm", "docs/changed [draft].md");
  git("commit", "-m", "Remove the document");
  git("switch", "main");
  git("merge", "--no-ff", "feature");
  assert.deepEqual(selectedInputs(directory, "pull_request"), []);
});

test("pushes and manual runs use the full documentation patterns", () => {
  for (const event of ["push", "workflow_dispatch"]) {
    assert.deepEqual(selectedInputs(fixture, event), [
      "README.md",
      "docs/**/*.md",
      "plugins/*/skills/**/*.md",
      "packages/*/README.md",
      "crates/*/README.md",
      "AGENTS.md",
    ]);
  }
});
