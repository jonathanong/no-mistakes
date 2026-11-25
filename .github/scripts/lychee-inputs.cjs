const { execFileSync } = require("node:child_process");
const { isDeepStrictEqual } = require("node:util");

const sourcePatterns = [
  "README.md",
  "docs/**/*.md",
  "plugins/*/skills/**/*.md",
  "packages/*/README.md",
  "crates/*/README.md",
  "AGENTS.md",
];

const workflowPath = ".github/workflows/ci.yml";
const gatewayScript = ".github/scripts/lychee-gateway.cjs";
const configPaths = [
  ".lycheeignore",
  workflowPath,
  ".github/scripts/lychee-inputs.cjs",
  gatewayScript,
  ".github/scripts/lychee-github-gateway.cjs",
];

// The PR merge checkout's first parent is the base tree.
const pullRequestRange = ["HEAD^1", "HEAD"];

function gitDiff(format, extraArgs, pathspecs = []) {
  const args = ["diff", format, "--no-renames", "-z", ...extraArgs, ...pullRequestRange];
  if (pathspecs.length > 0) args.push("--", ...pathspecs);
  const paths = execFileSync("git", args);
  const names = new TextDecoder("utf-8", { fatal: true }).decode(paths);
  if (/[\r\n]/u.test(names)) throw new Error("Lychee input paths must fit on one line");
  return names.split("\0").filter(Boolean);
}

function workflowLinkArguments(revision) {
  const input = execFileSync("git", ["show", `${revision}:${workflowPath}`]);
  const workflow = JSON.parse(
    execFileSync("yq", ["eval", "--output-format=json", "explode(.)", "-"], {
      input,
      encoding: "utf8",
    }),
  );
  const job = workflow?.jobs?.["quality-links"];
  const jobArgs = job?.env?.LYCHEE_ARGS ?? workflow?.env?.LYCHEE_ARGS;
  return (job?.steps ?? []).flatMap((step) => {
    const envArgs = step.env?.LYCHEE_ARGS ?? jobArgs;
    if (step.uses?.startsWith("lycheeverse/lychee-action@")) {
      const args = step.with?.args;
      return [{ args, envArgs: /\bLYCHEE_ARGS\b/iu.test(args) ? envArgs : undefined }];
    }
    if (step.run?.includes(gatewayScript)) return [{ retryArgs: envArgs }];
    return [];
  });
}

function configChanged() {
  const changes = gitDiff("--name-status", [], configPaths);
  if (changes.length === 0) return false;
  if (changes.length !== 2 || changes[0] !== "M" || changes[1] !== workflowPath) return true;
  const [base, head] = pullRequestRange.map(workflowLinkArguments);
  return !isDeepStrictEqual(base, head);
}

function hasDeletedOrRenamedFiles() {
  return gitDiff("--name-only", ["--diff-filter=D"]).length > 0;
}

function pullRequestSources(offline) {
  if (configChanged()) return offline ? [] : sourcePatterns;
  if (offline) return hasDeletedOrRenamedFiles() ? sourcePatterns : [];
  return gitDiff(
    "--name-only",
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
