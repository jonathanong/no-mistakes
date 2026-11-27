const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { existsSync, mkdtempSync, readFileSync, rmSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");

const root = path.resolve(__dirname, "../..");
const workflow = readWorkflow(path.join(root, ".github/workflows/release.yml"));
const step = workflow.jobs.prepare.steps.find(({ id }) => id === "bump");
const commands = [
  "pnpm() { :; }",
  "cargo() { :; }",
  'node() { if [[ "$1" == -p ]]; then echo 1.2.3; fi; }',
  'git() { case "$1" in rev-parse) echo release-sha ;; push) printf "%s\\n" "$*" >&3; printf "%s\\n" "$TEST_PUSH_OUTPUT" >&2; return "$TEST_PUSH_STATUS" ;; esac; }',
  "export -f git",
  step.run.replace("${{ inputs.bump }}", "patch"),
].join("\n");

const rerun = /::error::main moved[^\n]*Re-run the release workflow from current main/;
const token = /::error::The release GitHub App could not push[^\n]*bypass actor on main/;
const existingTag = "!\trefs/tags/v1.2.3:refs/tags/v1.2.3\t[rejected] (already exists)";
const mainRejected = "!\tHEAD:refs/heads/main\t[rejected]";

for (const [name, status, output, diagnosis] of [
  ["success", 0, "Done"],
  ["main advanced", 1, `${mainRejected} (fetch first)`, rerun],
  ["main diverged", 1, `${mainRejected} (non-fast-forward)`, rerun],
  [
    "HTTP 403",
    128,
    "fatal: unable to access 'https://example.com/repo/': The requested URL returned error: 403",
    token,
  ],
  [
    "permission denied",
    128,
    "remote: Permission to example/repo.git denied to release-bot.",
    token,
  ],
  [
    "authentication failed",
    128,
    "fatal: Authentication failed for 'https://example.com/repo/'",
    token,
  ],
  [
    "GitHub branch protection",
    1,
    "remote: error: GH006: Protected branch update failed for refs/heads/main.\nremote: error: Changes must be made through a pull request.",
    token,
  ],
  [
    "repository ruleset",
    1,
    "remote: error: GH013: Repository rule violations found for refs/heads/main.        \nremote: - Changes must be made through a pull request.",
    token,
  ],
  [
    "protected branch hook",
    1,
    "!\tHEAD:refs/heads/main\t[remote rejected] (protected branch hook declined)",
    token,
  ],
  ["fetch first outside main", 1, "!\tHEAD:refs/heads/other\t[rejected] (fetch first)"],
  [
    "tag already exists",
    1,
    `!\tHEAD:refs/heads/main\t[rejected] (atomic push failed)\n${existingTag}`,
  ],
  [
    "network failure",
    128,
    "fatal: unable to access 'https://example.com/repo/': Could not resolve host: example.com",
  ],
]) {
  test(`prepare release commit reports ${name}`, (t) => {
    const directory = mkdtempSync(path.join(os.tmpdir(), "release-push-"));
    t.after(() => rmSync(directory, { recursive: true, force: true }));
    const githubOutput = path.join(directory, "output");
    const result = spawnSync("bash", ["-e", "-o", "pipefail", "-c", commands], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        RESUME_VERSION: "",
        GITHUB_OUTPUT: githubOutput,
        TEST_PUSH_STATUS: String(status),
        TEST_PUSH_OUTPUT: output,
      },
      stdio: ["ignore", "pipe", "pipe", "pipe"],
      timeout: 5000,
    });

    assert.ifError(result.error);
    assert.ok(`${result.stdout}${result.stderr}`.includes(output), result.stderr);
    if (diagnosis) assert.match(result.stderr, diagnosis);
    else assert.doesNotMatch(result.stderr, /::error::/);
    assert.equal(result.status, status, result.stderr);
    assert.equal(result.output[3], "push --atomic --porcelain origin HEAD:main v1.2.3\n");
    if (status === 0)
      assert.equal(readFileSync(githubOutput, "utf8"), "release-sha=release-sha\nversion=1.2.3\n");
    else assert.equal(existsSync(githubOutput), false);
  });
}
