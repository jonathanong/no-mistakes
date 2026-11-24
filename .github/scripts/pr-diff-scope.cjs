// Shared git diff helpers for pull-request path scopes. See docs/development/ci-builds.md.

function diffBase(eventName, base, head) {
  if (!/^[0-9a-f]{40}$/.test(base) || !/^[0-9a-f]{40}$/.test(head)) {
    throw new Error("Expected full commit SHAs");
  }
  // Pushes compare the event endpoints. A pull request compares the merge base
  // so files that changed only on the base branch do not count.
  if (eventName !== "pull_request") return base;
  const result = require("node:child_process").spawnSync("git", ["merge-base", base, head], {
    encoding: "utf8",
  });
  const sha = (result.stdout || "").trim();
  if (result.status !== 0 || !/^[0-9a-f]{40}$/.test(sha)) {
    throw new Error("Expected a merge base for the pull request");
  }
  return sha;
}

function readChangedFiles() {
  const fields = require("node:fs").readFileSync(0).toString("utf8").split("\0");
  if (fields.pop() !== "" || fields.length % 2 !== 0) {
    throw new Error("Expected NUL-terminated git diff name-status pairs");
  }
  const changes = [];
  for (let index = 0; index < fields.length; index += 2) {
    const status = fields[index];
    const path = fields[index + 1];
    if (!/^[ADMT]$/.test(status) || !path) {
      throw new Error("Expected added, deleted, modified, or type-changed git diff path");
    }
    changes.push({ status, path });
  }
  return changes;
}

if (require.main === module) {
  const diffBaseFlag = process.argv.indexOf("--diff-base");
  if (diffBaseFlag !== -1) {
    const [eventName, base, head] = process.argv.slice(diffBaseFlag + 1);
    if (!eventName || !base || !head)
      throw new Error("Expected event name, base SHA, and head SHA");
    process.stdout.write(`${diffBase(eventName, base, head)}\n`);
    process.exit(0);
  }
  throw new Error("usage: pr-diff-scope.cjs --diff-base <event> <base> <head>");
}

module.exports = { diffBase, readChangedFiles };
