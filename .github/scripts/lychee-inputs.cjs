const { execFileSync } = require("node:child_process");

const sourcePatterns = [
  "README.md",
  "docs/**/*.md",
  "plugins/*/skills/**/*.md",
  "packages/*/README.md",
  "crates/*/README.md",
  "AGENTS.md",
];

const configPaths = [
  ".lycheeignore",
  ".github/workflows/ci.yml",
  ".github/scripts/lychee-inputs.cjs",
  ".github/scripts/lychee-gateway.cjs",
  ".github/scripts/lychee-github-gateway.cjs",
];

function gitDiffNames(extraArgs, pathspecs = []) {
  // The PR merge checkout's first parent is the base tree.
  const args = ["diff", "--name-only", "--no-renames", "-z", ...extraArgs, "HEAD^1", "HEAD"];
  if (pathspecs.length > 0) args.push("--", ...pathspecs);
  const paths = execFileSync("git", args);
  const names = new TextDecoder("utf-8", { fatal: true }).decode(paths);
  if (/[\r\n]/u.test(names)) throw new Error("Lychee input paths must fit on one line");
  return names.split("\0").filter(Boolean);
}

function configChanged() {
  return gitDiffNames([], configPaths).length > 0;
}

function hasDeletedOrRenamedFiles() {
  return gitDiffNames(["--diff-filter=D"]).length > 0;
}

function pullRequestSources(offline) {
  if (configChanged()) return offline ? [] : sourcePatterns;
  if (offline) return hasDeletedOrRenamedFiles() ? sourcePatterns : [];
  return gitDiffNames(
    ["--diff-filter=AMT"],
    sourcePatterns.map((pattern) => `:(glob)${pattern}`),
  ).map((name) => name.replace(/[[\]*?]/gu, "[$&]"));
}

const offline = process.argv.includes("--offline");
const inputs =
  process.env.GITHUB_EVENT_NAME === "pull_request"
    ? pullRequestSources(offline)
    : offline
      ? []
      : sourcePatterns;
if (inputs.length > 0) process.stdout.write(`${inputs.join("\n")}\n`);
