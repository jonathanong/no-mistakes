const fs = require("node:fs");

const BUDGET = 8 * 1024 ** 3;
const MAX_PAGES = 10;
function sharedIdentity(key) {
  if (key.startsWith("v0-rust-rust-quality-release-check-v1-")) return undefined;
  return key.match(
    /^v\d+-rust-(linux-runtime-v1|rust-quality-[\w-]+-v1|packages-packages)-(Linux-x64)-([a-f0-9]{8,64})-[a-f0-9]{8,64}$/,
  );
}
function handoff(key) {
  return key.match(
    /^codspeed-handoff-v1-(Linux|macOS|Windows)-(X64|ARM64)-([1-9]\d*)-[1-9]\d*-[a-f0-9]{40}-[a-f0-9]{64}$/,
  );
}
function workspace(key, version) {
  return key.match(
    new RegExp(
      `^(linux-runtime-workspace-v${version}-(?:cli|addon)|native-(?:runtime|test-harness)-workspace-v${version}-(?:x86_64-pc-windows-msvc|aarch64-apple-darwin)|(?:standalone-lib|coverage)-workspace-v${version}-Linux-X64)-`,
    ),
  );
}
function family(key, common = new Set()) {
  const pnpm = key.match(
    /^(pnpm-cache-(?:Linux|macOS|Windows)-(?:x64|arm64)-[a-f0-9]{64}-[a-f0-9]{64}-[a-f0-9]{64})-[1-9]\d*-[1-9]\d*-[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/,
  );
  if (pnpm) return { name: pnpm[1], preferred: 1, largest: true };
  const transport = handoff(key);
  if (transport) return { name: `benchmark-handoff:${transport[1]}:${transport[2]}`, preferred: 1 };
  const registry = key.match(
    /^v\d+-rust-cargo-registry-v1-[a-f0-9]{64}-(Linux|Darwin|Windows_NT)-(x64|arm64)$/,
  );
  if (registry) return { name: `registry:${registry[1]}:${registry[2]}`, preferred: 1 };
  const benchmark = key.match(
    /^v\d+-rust-(bench-(?:ARM64|X64)-bench)-(Linux-(?:arm64|x64))-([a-f0-9]{8,64})-[a-f0-9]{8,64}$/,
  );
  if (benchmark)
    return { name: `dependencies:${benchmark[1]}:${benchmark[2]}:${benchmark[3]}`, preferred: 1 };
  let match = workspace(key, 2);
  if (match) return { name: match[1], preferred: 1 };
  match = sharedIdentity(key);
  if (match) {
    const identity = `${match[2]}:${match[3]}`;
    const canonical = match[1] === "linux-runtime-v1";
    const scope = canonical || common.has(identity) ? "shared-linux" : match[1];
    return { name: `dependencies:${scope}:${identity}`, preferred: Number(canonical) };
  }
  match = key.match(
    /^v\d+-rust-((?:native-runtime-v2|native-test-harness-v2)-(?:aarch64-apple-darwin|x86_64-pc-windows-msvc)|coverage-nextest-v1|standalone-lib-v2|rust-doctests-v1|rust-quality-release-check-v1)-([\w]+-[\w]+)-([a-f0-9]{8,64})-[a-f0-9]{8,64}$/,
  );
  if (match) return { name: `dependencies:${match[1]}:${match[2]}:${match[3]}`, preferred: 1 };
  return undefined;
}
function obsoleteDependency(key) {
  return /^v\d+-rust-((?:native-runtime-v2|native-test-harness-v2)-(?:aarch64-apple-darwin|x86_64-pc-windows-msvc)|coverage-nextest-v1|standalone-lib-v2)-[\w]+-[\w]+-[a-f0-9]{8,64}-[a-f0-9]{8,64}$/.test(
    key,
  );
}
function plan(caches, states, budget = BUDGET, sourceRunStates = {}) {
  const common = new Set(
    caches
      .filter((cache) => cache.ref === "refs/heads/main")
      .map((cache) => sharedIdentity(cache.key))
      .filter((match) => match?.[1] === "linux-runtime-v1")
      .map((match) => `${match[2]}:${match[3]}`),
  );
  const groups = new Map();
  const remove = [];
  let before = 0;
  const seen = new Set();
  for (const cache of caches) {
    if (
      !Number.isSafeInteger(cache.id) ||
      cache.id < 1 ||
      seen.has(cache.id) ||
      !Number.isSafeInteger(cache.size_in_bytes) ||
      cache.size_in_bytes < 0 ||
      typeof cache.key !== "string"
    )
      throw new Error("Invalid or duplicate cache metadata");
    seen.add(cache.id);
    before += cache.size_in_bytes;
    const pr = cache.ref?.match(/^refs\/pull\/([1-9]\d*)\/(?:merge|head)$/);
    if (pr) {
      if (states[pr[1]] === "closed")
        remove.push({ ...cache, reason: `closed pull request ${pr[1]}` });
      continue;
    }
    if (cache.ref !== "refs/heads/main") continue;
    if (obsoleteDependency(cache.key) || workspace(cache.key, 1)) {
      remove.push({ ...cache, reason: "retired producer cache generation" });
      continue;
    }
    const transport = handoff(cache.key);
    if (transport && sourceRunStates[transport[3]] !== "completed") continue;
    const identity = family(cache.key, common);
    if (!identity) continue;
    const created = Date.parse(cache.created_at);
    if (!Number.isFinite(created)) throw new Error(`Missing cache creation time: ${cache.id}`);
    if (!groups.has(identity.name)) groups.set(identity.name, []);
    groups.get(identity.name).push({ ...cache, ...identity, created });
  }
  const kept = [];
  for (const [name, versions] of groups) {
    versions.sort(
      (a, b) =>
        b.preferred - a.preferred ||
        (a.largest ? b.size_in_bytes - a.size_in_bytes : 0) ||
        b.created - a.created ||
        b.id - a.id,
    );
    kept.push({ family: name, id: versions[0].id, key: versions[0].key });
    for (const cache of versions.slice(1)) remove.push({ ...cache, reason: `superseded ${name}` });
  }
  remove.sort((a, b) => a.id - b.id);
  if (!Number.isSafeInteger(before)) throw new Error("Cache size exceeds safe integer range");
  let after = before - remove.reduce((sum, cache) => sum + cache.size_in_bytes, 0);
  const removed = new Set(remove.map((cache) => cache.id));
  const expendable = caches.filter(
    (cache) =>
      cache.ref === "refs/heads/main" &&
      workspace(cache.key, 2) &&
      !removed.has(cache.id) &&
      Number.isFinite(Date.parse(cache.last_accessed_at)),
  );
  expendable.sort(
    (a, b) =>
      Date.parse(a.last_accessed_at) - Date.parse(b.last_accessed_at) ||
      Date.parse(a.created_at) - Date.parse(b.created_at) ||
      a.id - b.id,
  );
  for (const cache of expendable) {
    if (after <= budget) break;
    remove.push({ ...cache, reason: "budget-workspace" });
    removed.add(cache.id);
    after -= cache.size_in_bytes;
  }
  remove.sort((a, b) => a.id - b.id);
  return {
    before,
    after,
    budget,
    withinBudget: after <= budget,
    remove,
    kept: kept.filter((cache) => !removed.has(cache.id)),
  };
}
function trustedEvent(name, event, repository, ref) {
  if (name === "push")
    return (
      ref === "refs/heads/main" &&
      event.ref === "refs/heads/main" &&
      event.repository?.full_name === repository
    );
  if (name === "workflow_dispatch") return ref === "refs/heads/main";
  if (name === "pull_request_target")
    return (
      event.action === "closed" &&
      event.pull_request?.state === "closed" &&
      event.pull_request.base?.ref === "main" &&
      event.pull_request.base?.repo?.full_name === repository
    );
  const run = event.workflow_run;
  return (
    name === "workflow_run" &&
    run?.name === "Test CI" &&
    run.event === "push" &&
    run.head_branch === "main" &&
    run.repository?.full_name === repository &&
    run.path === ".github/workflows/ci.yml"
  );
}
async function collect(request) {
  const caches = [];
  const seen = new Set();
  for (let page = 1; page <= MAX_PAGES; page++) {
    const result = await request("GET", `/actions/caches?per_page=100&page=${page}`);
    if (
      !Array.isArray(result.actions_caches) ||
      result.actions_caches.length > 100 ||
      !Number.isSafeInteger(result.total_count) ||
      result.total_count > MAX_PAGES * 100
    )
      throw new Error("Cache inventory incomplete; budget unknown (maximum 1000 caches)");
    for (const cache of result.actions_caches) {
      if (seen.has(cache.id))
        throw new Error("Cache inventory changed while paging; budget unknown");
      seen.add(cache.id);
      caches.push(cache);
    }
    if (caches.length >= result.total_count || result.actions_caches.length < 100) {
      if (caches.length !== result.total_count)
        throw new Error("Cache inventory incomplete; budget unknown");
      return caches;
    }
  }
  throw new Error("Cache inventory incomplete; budget unknown");
}
async function retain(request, apply = false) {
  const caches = await collect(request);
  const prs = [
    ...new Set(
      caches.map((cache) => cache.ref?.match(/^refs\/pull\/(\d+)\//)?.[1]).filter(Boolean),
    ),
  ];
  if (prs.length > 100)
    throw new Error("Pull request inventory exceeds 100 lookups; budget unknown");
  const states = {};
  for (const pr of prs) {
    const result = await request("GET", `/pulls/${pr}`);
    if (!["open", "closed"].includes(result.state))
      throw new Error(`Pull request ${pr} state is unknown; preserving caches`);
    states[pr] = result.state;
  }
  const runs = [
    ...new Set(
      caches
        .filter((cache) => cache.ref === "refs/heads/main")
        .map((cache) => handoff(cache.key)?.[3])
        .filter(Boolean),
    ),
  ];
  if (runs.length > 100)
    throw new Error("Handoff inventory exceeds 100 source-run lookups; budget unknown");
  const sourceRunStates = {};
  for (const run of runs) {
    try {
      const result = await request("GET", `/actions/runs/${run}`);
      sourceRunStates[run] =
        String(result.id) === run && result.status === "completed" ? "completed" : "unknown";
    } catch (error) {
      if (error.status !== 404) throw error;
      sourceRunStates[run] = "unknown";
    }
  }
  const result = plan(caches, states, BUDGET, sourceRunStates);
  if (apply)
    for (const cache of result.remove) await request("DELETE", `/actions/caches/${cache.id}`);
  return { ...result, applied: apply };
}
function summary(result) {
  const gib = (bytes) => (bytes / 1024 ** 3).toFixed(3);
  return `## Cache retention\n\nSnapshot: ${gib(result.before)} GiB. Projected after ${result.applied ? "deletions" : "planned deletions"}: ${gib(result.after)} GiB; budget ${gib(result.budget)} GiB.\n\n${result.remove.length} obsolete records (${result.remove.filter((cache) => cache.reason === "budget-workspace").length} budget-workspace evictions); ${result.kept.length} current main families retained. Open pull request records are preserved.\n\n${result.withinBudget ? "Within snapshot budget." : "Budget remains exceeded; remaining families, open pull requests and unrecognized records are protected."} New saves after the listing are not included.\n`;
}
async function main(env = process.env) {
  if (process.argv.slice(2).some((arg) => arg !== "--apply"))
    throw new Error("Usage: cache-retention.cjs [--apply]");
  const repository = env.GITHUB_REPOSITORY;
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository || "") || !env.GH_TOKEN)
    throw new Error("GITHUB_REPOSITORY and GH_TOKEN are required");
  const event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  if (!trustedEvent(env.GITHUB_EVENT_NAME, event, repository, env.GITHUB_REF))
    throw new Error(
      "Cache retention requires a trusted main workflow, closed base pull request or main dispatch",
    );
  const request = async (method, route) => {
    const response = await fetch(`https://api.github.com/repos/${repository}${route}`, {
      method,
      headers: {
        Authorization: `Bearer ${env.GH_TOKEN}`,
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2026-03-10",
        "User-Agent": "no-mistakes-cache-retention",
      },
      signal: AbortSignal.timeout(20_000),
    });
    if (method === "DELETE" && [204, 404].includes(response.status)) return {};
    if (!response.ok)
      throw Object.assign(
        new Error(
          `GitHub cache API ${method} failed with HTTP ${response.status}; retention incomplete, budget unknown`,
        ),
        { status: response.status },
      );
    return response.json();
  };
  const result = await retain(request, process.argv.includes("--apply"));
  const report = summary(result);
  if (env.GITHUB_STEP_SUMMARY) fs.appendFileSync(env.GITHUB_STEP_SUMMARY, report);
  console.log(report);
}
function failureSummary(message) {
  return `## Cache retention incomplete\n\n${message}\n\nBudget unknown; retention did not complete.\n`;
}
if (require.main === module)
  main().catch((error) => {
    if (process.env.GITHUB_STEP_SUMMARY)
      fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, failureSummary(error.message));
    console.error(error.message);
    process.exitCode = 1;
  });
module.exports = { plan, family, collect, retain, trustedEvent, summary, failureSummary, BUDGET };
