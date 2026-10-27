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

function resolveRefs(
  api,
  compiler,
  repository,
  harnessCommit,
  candidateSha = "",
  now = new Date(),
) {
  validateCompiler(compiler);
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository) || !SHA.test(harnessCommit)) {
    throw new Error("Invalid repository or measurement harness commit");
  }
  const endpoint = `repos/${repository}/releases/latest`;
  if (candidateSha && !SHA.test(candidateSha)) {
    throw new Error("Candidate SHA must be an exact 40-character lowercase commit SHA");
  }
  const candidateEndpoint = candidateSha
    ? `repos/${repository}/commits/${candidateSha}`
    : `repos/${repository}/commits/main`;
  const candidateIdentity = commitIdentity(api(candidateEndpoint));
  if (candidateSha && candidateIdentity.commit !== candidateSha) {
    throw new Error("GitHub candidate commit does not match the requested SHA");
  }
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
    candidate: candidateSha
      ? { kind: "commit", ref: candidateSha, ...candidateIdentity }
      : { kind: "main", ref: "refs/heads/main", ...candidateIdentity },
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
    `Candidate: ${refs.candidate.kind === "main" ? "main" : "commit"} \`${refs.candidate.commit}\`.`,
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
    process.env.HOSTED_CANDIDATE_SHA || "",
  );
  writeFileSync(output, `${JSON.stringify(refs, null, 2)}\n`);
  for (const key of ["candidate", "baseline"]) {
    appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${refs[key].commit}\n`);
  }
  appendFileSync(process.env.GITHUB_OUTPUT, `compiler=${refs.compiler}\n`);
  if (process.env.GITHUB_STEP_SUMMARY)
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, sourceSummary(refs));
}

module.exports = { resolveRefs, validateCompiler, commitIdentity, sourceSummary };
