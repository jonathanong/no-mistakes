// Resolves the git diff base for workflow path-scoping steps.
// Pull requests keep their PR base SHA unchanged. Push runs use the head SHA
// of the most recent successful run of the same workflow on the same branch so
// commits skipped by ref-keyed concurrency stay in scope.
// Any missing, non-ancestor, or unfetchable base falls back to empty (full scope).

const { spawnSync } = require("node:child_process");

const SHA_RE = /^[0-9a-f]{40}$/;
const ZERO_SHA = "0000000000000000000000000000000000000000";
const DEFAULT_MAX_DEEPEN_ATTEMPTS = 4;
const DEFAULT_DEEPEN_STEP = 50;

function isValidSha(value) {
  return typeof value === "string" && SHA_RE.test(value) && value !== ZERO_SHA;
}

function selectPreviousRunBase(payload, currentRunId) {
  const runs = Array.isArray(payload?.workflow_runs) ? payload.workflow_runs : [];
  const currentId = currentRunId == null ? "" : String(currentRunId);
  for (const run of runs) {
    if (!run || typeof run !== "object") continue;
    if (currentId && String(run.id) === currentId) continue;
    if (run.status !== "completed") continue;
    if (run.conclusion !== "success") continue;
    if (!isValidSha(run.head_sha)) continue;
    return run.head_sha;
  }
  return "";
}

function fetchWorkflowRuns({ repo, workflow, branch, env = process.env }) {
  const endpoint =
    `repos/${repo}/actions/workflows/${encodeURIComponent(workflow)}/runs` +
    `?branch=${encodeURIComponent(branch)}&event=push&status=completed&per_page=30`;
  const result = spawnSync("gh", ["api", endpoint], { encoding: "utf8", env });
  if (result.status !== 0) {
    return {
      ok: false,
      reason: `gh api failed: ${(result.stderr || "").trim() || "unknown error"}`,
    };
  }
  try {
    return { ok: true, payload: JSON.parse(result.stdout || "{}") };
  } catch (error) {
    return { ok: false, reason: `invalid gh api JSON: ${error.message}` };
  }
}

function ensureAncestor({
  baseSha,
  headSha,
  cwd,
  maxDeepenAttempts = DEFAULT_MAX_DEEPEN_ATTEMPTS,
  deepenStep = DEFAULT_DEEPEN_STEP,
}) {
  if (!isValidSha(baseSha) || !isValidSha(headSha)) {
    return { ok: false, reason: "invalid base or head SHA" };
  }
  const git = (args) => spawnSync("git", args, { encoding: "utf8", cwd });
  const hasCommit = (sha) => git(["cat-file", "-e", `${sha}^{commit}`]).status === 0;

  if (!hasCommit(headSha)) {
    if (git(["fetch", "--no-tags", "--depth=1", "origin", headSha]).status !== 0) {
      return { ok: false, reason: `head commit ${headSha} could not be fetched` };
    }
  }
  if (!hasCommit(baseSha)) {
    if (git(["fetch", "--no-tags", "--depth=1", "origin", baseSha]).status !== 0) {
      return { ok: false, reason: `base commit ${baseSha} could not be fetched` };
    }
  }
  if (git(["merge-base", "--is-ancestor", baseSha, headSha]).status === 0) {
    return { ok: true };
  }

  const shallow = git(["rev-parse", "--is-shallow-repository"]).stdout?.trim() === "true";
  if (shallow) {
    for (let attempt = 0; attempt < maxDeepenAttempts; attempt += 1) {
      if (git(["fetch", "--no-tags", `--deepen=${deepenStep}`, "origin", headSha]).status !== 0) {
        break;
      }
      if (git(["merge-base", "--is-ancestor", baseSha, headSha]).status === 0) {
        return { ok: true };
      }
    }
  }
  return { ok: false, reason: `base commit ${baseSha} is not an ancestor of ${headSha}` };
}

function resolveDiffBase(options) {
  const { event, workflow, repo, branch, head, prBase, currentRunId, runsPayload, cwd, env } =
    options;

  if (event === "pull_request") {
    return isValidSha(prBase)
      ? { baseSha: prBase, reason: "pull_request base" }
      : { baseSha: "", reason: "missing pull_request base" };
  }
  if (event !== "push") {
    return { baseSha: "", reason: `event ${event} uses full scope` };
  }

  let payload = runsPayload;
  if (payload === undefined) {
    const fetched = fetchWorkflowRuns({ repo, workflow, branch, env });
    if (!fetched.ok) return { baseSha: "", reason: fetched.reason };
    payload = fetched.payload;
  }

  const candidate = selectPreviousRunBase(payload, currentRunId);
  if (!candidate) {
    return { baseSha: "", reason: "no prior successful push run found" };
  }

  const ancestry = ensureAncestor({ baseSha: candidate, headSha: head, cwd });
  if (!ancestry.ok) {
    return { baseSha: "", reason: ancestry.reason };
  }
  return { baseSha: candidate, reason: "previous successful push run" };
}

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (!flag.startsWith("--")) {
      throw new Error(`Unexpected argument: ${flag}`);
    }
    const value = argv[index + 1];
    if (value === undefined) {
      throw new Error(`Missing value for ${flag}`);
    }
    index += 1;
    switch (flag) {
      case "--event":
        options.event = value;
        break;
      case "--workflow":
        options.workflow = value;
        break;
      case "--repo":
        options.repo = value;
        break;
      case "--branch":
        options.branch = value;
        break;
      case "--head":
        options.head = value;
        break;
      case "--pr-base":
        options.prBase = value;
        break;
      case "--current-run-id":
        options.currentRunId = value;
        break;
      default:
        throw new Error(`Unknown flag: ${flag}`);
    }
  }
  return options;
}

if (require.main === module) {
  const options = parseArgs(process.argv.slice(2));
  if (!options.event) {
    throw new Error(
      "usage: push-diff-base.cjs --event <name> --workflow <file> --repo <owner/repo> --branch <name> --head <sha>",
    );
  }
  if (options.event === "push" && (!options.repo || !options.workflow || !options.branch)) {
    throw new Error("push event requires --repo, --workflow, and --branch");
  }
  const result = resolveDiffBase(options);
  if (!result.baseSha && options.event === "push") {
    process.stderr.write(`push-diff-base: ${result.reason}; falling back to full scope\n`);
  }
  process.stdout.write(`${result.baseSha}\n`);
}

module.exports = { ensureAncestor, isValidSha, resolveDiffBase, selectPreviousRunBase };
