const { execFileSync } = require("node:child_process");

const sourcePatterns = [
  "README.md",
  "docs/**/*.md",
  "plugins/*/skills/**/*.md",
  "packages/*/README.md",
  "crates/*/README.md",
  "AGENTS.md",
];

function pullRequestSources() {
  // The PR merge checkout's first parent is the base tree.
  const paths = execFileSync("git", [
    "diff",
    "--name-only",
    "--no-renames",
    "--diff-filter=AMT",
    "-z",
    "HEAD^1",
    "HEAD",
    "--",
    ...sourcePatterns.map((pattern) => `:(glob)${pattern}`),
  ]);
  const names = new TextDecoder("utf-8", { fatal: true }).decode(paths);
  if (/[\r\n]/u.test(names)) throw new Error("Lychee input paths must fit on one line");
  return names
    .split("\0")
    .filter(Boolean)
    .map((name) => name.replace(/[[\]*?]/gu, "[$&]"));
}

const inputs =
  process.env.GITHUB_EVENT_NAME === "pull_request" ? pullRequestSources() : sourcePatterns;
if (inputs.length > 0) process.stdout.write(`${inputs.join("\n")}\n`);
