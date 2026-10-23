const { spawnSync } = require("node:child_process");
const { appendFileSync, writeFileSync } = require("node:fs");

const SHA = /^[0-9a-f]{40}$/;
const STABLE = /^v?\d+\.\d+\.\d+$/;

function validateCompiler(compiler) {
  if (!/^1\.\d+\.\d+$/.test(compiler)) throw new Error("Expected an exact stable Rust compiler");
  return compiler;
}

function commitIdentity(response) {
  if (!SHA.test(response.sha) || !SHA.test(response.commit?.tree?.sha)) {
    throw new Error("GitHub did not return an immutable commit and tree");
  }
  return { commit: response.sha, tree: response.commit.tree.sha };
}

function validateCandidateSource(candidate, repository) {
  if (candidate?.kind === "main" && candidate.ref === "refs/heads/main") return;
  const pr = candidate?.pullRequest;
  if (
    candidate?.kind !== "pull-request" ||
    candidate.event !== "workflow_dispatch" ||
    !Number.isSafeInteger(pr?.number) ||
    pr.number <= 0 ||
    candidate.ref !== `refs/pull/${pr.number}/head` ||
    pr.url !== `https://github.com/${repository}/pull/${pr.number}` ||
    pr.headRepository !== repository ||
    pr.baseRepository !== repository ||
    pr.baseRef !== "main" ||
    typeof pr.headRef !== "string" ||
    !pr.headRef ||
    /\s/.test(pr.headRef) ||
    !SHA.test(pr.expectedHeadCommit) ||
    candidate.commit !== pr.expectedHeadCommit
  )
    throw new Error(
      "Candidate source must be main or an exact manual same-repository pull request",
    );
}

function candidateSource(api, repository, selection) {
  const { source = "main", eventName, pullRequest = "", expectedHead = "" } = selection;
  if (source === "main") {
    if (pullRequest || expectedHead) throw new Error("PR inputs require pull-request source");
    return {
      kind: "main",
      ref: "refs/heads/main",
      ...commitIdentity(api(`repos/${repository}/commits/main`)),
    };
  }
  if (
    source !== "pull-request" ||
    eventName !== "workflow_dispatch" ||
    !/^[1-9][0-9]*$/.test(pullRequest) ||
    !Number.isSafeInteger(Number(pullRequest)) ||
    !SHA.test(expectedHead)
  )
    throw new Error("Expected manual pull-request source, positive PR number and exact head SHA");
  const number = Number(pullRequest);
  const pr = api(`repos/${repository}/pulls/${number}`);
  if (
    pr?.number !== number ||
    pr.state !== "open" ||
    pr.html_url !== `https://github.com/${repository}/pull/${number}` ||
    pr.head?.repo?.full_name !== repository ||
    pr.base?.repo?.full_name !== repository ||
    pr.base?.ref !== "main" ||
    pr.head?.sha !== expectedHead
  )
    throw new Error("Expected open same-repository PR targeting main at the exact requested head");
  const candidate = {
    kind: "pull-request",
    ref: `refs/pull/${number}/head`,
    event: eventName,
    ...commitIdentity(api(`repos/${repository}/commits/${expectedHead}`)),
    pullRequest: {
      number,
      url: pr.html_url,
      headRepository: pr.head.repo.full_name,
      headRef: pr.head.ref,
      baseRepository: pr.base.repo.full_name,
      baseRef: pr.base.ref,
      expectedHeadCommit: expectedHead,
    },
  };
  validateCandidateSource(candidate, repository);
  return candidate;
}

function resolveRefs(api, compiler, repository, harnessCommit, now = new Date(), selection = {}) {
  validateCompiler(compiler);
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository) || !SHA.test(harnessCommit)) {
    throw new Error("Invalid repository or measurement harness commit");
  }
  const endpoint = `repos/${repository}/releases/latest`;
  const candidate = candidateSource(api, repository, selection);
  const release = api(endpoint);
  if (
    release.draft !== false ||
    release.prerelease !== false ||
    !STABLE.test(release.tag_name) ||
    !Number.isSafeInteger(release.id) ||
    release.id <= 0 ||
    release.html_url !== `https://github.com/${repository}/releases/tag/${release.tag_name}`
  ) {
    throw new Error("Expected GitHub's latest published stable release");
  }
  const baseline = api(`repos/${repository}/commits/${encodeURIComponent(release.tag_name)}`);
  return {
    schemaVersion: 1,
    repository,
    harnessCommit,
    compiler,
    resolvedAt: now.toISOString(),
    candidate,
    baseline: {
      kind: "release",
      ref: `refs/tags/${release.tag_name}`,
      ...commitIdentity(baseline),
      release: { id: release.id, tag: release.tag_name, url: release.html_url, endpoint },
    },
  };
}

function githubApi(endpoint) {
  const result = spawnSync("gh", ["api", "--method", "GET", endpoint], {
    encoding: "utf8",
    timeout: 90_000,
  });
  if (result.error || result.status !== 0)
    throw new Error(`GitHub ${endpoint}: ${result.error || result.stderr}`);
  return JSON.parse(result.stdout);
}

function sourceSummary(refs) {
  return [
    `Candidate: ${refs.candidate.kind === "main" ? "main" : `pull request #${refs.candidate.pullRequest.number}`} \`${refs.candidate.commit}\`.`,
    `Baseline: release [${refs.baseline.release.tag}](${refs.baseline.release.url}) \`${refs.baseline.commit}\`.`,
    ...(refs.baseline.commit === refs.candidate.commit
      ? [
          "> **Warning:** baseline and candidate resolve to the same commit; this run measures variance, not a change between versions. Absolute budgets still apply.",
        ]
      : []),
    "",
  ].join("\n\n");
}

if (require.main === module) {
  const [compiler, output] = process.argv.slice(2);
  const refs = resolveRefs(
    githubApi,
    compiler,
    process.env.GITHUB_REPOSITORY,
    process.env.HOSTED_HARNESS_COMMIT,
    new Date(),
    {
      source: process.env.HOSTED_CANDIDATE_SOURCE || "main",
      eventName: process.env.GITHUB_EVENT_NAME,
      pullRequest: process.env.HOSTED_PULL_REQUEST || "",
      expectedHead: process.env.HOSTED_EXPECTED_HEAD || "",
    },
  );
  writeFileSync(output, `${JSON.stringify(refs, null, 2)}\n`);
  for (const key of ["candidate", "baseline"]) {
    appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${refs[key].commit}\n`);
  }
  appendFileSync(process.env.GITHUB_OUTPUT, `compiler=${refs.compiler}\n`);
  if (process.env.GITHUB_STEP_SUMMARY)
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, sourceSummary(refs));
}

module.exports = {
  resolveRefs,
  validateCompiler,
  commitIdentity,
  sourceSummary,
  validateCandidateSource,
};
