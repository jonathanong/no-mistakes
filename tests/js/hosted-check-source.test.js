const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const {
  resolveRefs,
  sourceSummary,
  validateCandidateSource,
} = require("../../.github/scripts/hosted-check-refs.cjs");
const { validateBuildPair } = require("../../.github/scripts/hosted-check-provenance.cjs");

const repository = "jonathanong/no-mistakes";
const expectedHead = "1111111111111111111111111111111111111111";
const compiler = "1.99.1";
const harness = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const fixturePath = join(__dirname, "../../fixtures/hosted-check/pull-request-responses.json");
const savedResponses = JSON.parse(readFileSync(fixturePath, "utf8"));
const mainResponses = JSON.parse(
  readFileSync(join(__dirname, "../../fixtures/hosted-check/build/github-responses.json"), "utf8"),
);
const invalid = savedResponses.invalid;

function resolve(selection, patches = {}) {
  return resolveRefs(
    (endpoint) => {
      const response = patches[endpoint] ?? savedResponses[endpoint] ?? mainResponses[endpoint];
      if (!response) throw new Error(`Unexpected GitHub endpoint: ${endpoint}`);
      return response;
    },
    compiler,
    repository,
    harness,
    new Date("2026-10-23T00:00:00Z"),
    selection,
  );
}

const manualPullRequest = {
  source: "pull-request",
  eventName: "workflow_dispatch",
  pullRequest: "2585",
  expectedHead,
};

test("selects an open same-repository PR only at its explicitly expected head", () => {
  const refs = resolve(manualPullRequest);
  assert.equal(refs.candidate.kind, "pull-request");
  assert.equal(refs.candidate.ref, "refs/pull/2585/head");
  assert.equal(refs.candidate.commit, expectedHead);
  assert.equal(refs.candidate.tree, "2222222222222222222222222222222222222222");
  assert.equal(refs.candidate.pullRequest.expectedHeadCommit, expectedHead);
  assert.equal(refs.candidate.pullRequest.headRepository, repository);
  assert.equal(refs.candidate.pullRequest.baseRepository, repository);
  assert.equal(refs.candidate.pullRequest.baseRef, "main");
  assert.match(sourceSummary(refs), /pull request #2585/);
  assert.match(sourceSummary(refs), new RegExp(expectedHead));
  validateCandidateSource(refs.candidate, repository);
});

test("keeps main as the default and rejects PR selection outside a manual dispatch", () => {
  const main = resolve({});
  assert.equal(main.candidate.kind, "main");
  assert.equal(main.candidate.ref, "refs/heads/main");
  assert.throws(() => resolve({ ...manualPullRequest, source: "main" }), /PR inputs require/);
  assert.throws(
    () => resolve({ ...manualPullRequest, eventName: "schedule" }),
    /manual pull-request source/,
  );
  assert.throws(
    () => resolve({ ...manualPullRequest, source: "unknown" }),
    /manual pull-request source/,
  );
  assert.throws(() => resolve({ ...manualPullRequest, pullRequest: "0" }), /positive PR number/);
  assert.throws(
    () => resolve({ ...manualPullRequest, pullRequest: "9007199254740992" }),
    /positive PR number/,
  );
  assert.throws(() => resolve({ ...manualPullRequest, expectedHead: "main" }), /exact head SHA/);
});

test("production build provenance accepts matched PR refs and rejects forged or mixed metadata", () => {
  const builds = JSON.parse(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/source-pair.json"), "utf8"),
  );
  const candidate = resolve(manualPullRequest).candidate;
  for (const build of Object.values(builds)) {
    build.refs.candidate = candidate;
  }
  builds.candidate.commit = candidate.commit;
  builds.candidate.tree = candidate.tree;
  validateBuildPair(builds);

  const forged = structuredClone(builds);
  forged.baseline.refs.candidate = {
    ...forged.baseline.refs.candidate,
    pullRequest: {
      ...forged.baseline.refs.candidate.pullRequest,
      baseRepository: "someone/fork",
    },
  };
  forged.candidate.refs.candidate = structuredClone(forged.baseline.refs.candidate);
  assert.throws(() => validateBuildPair(forged), /Candidate source must be main or an exact/);

  const mixed = structuredClone(builds);
  mixed.candidate.refs.candidate = {
    ...mixed.candidate.refs.candidate,
    pullRequest: {
      ...mixed.candidate.refs.candidate.pullRequest,
      expectedHeadCommit: invalid.differentExpectedHead,
    },
  };
  assert.throws(() => validateBuildPair(mixed), /Variant source selections differ/);
});

test("workflow keeps PR selection opt-in and reuses the existing build matrix", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  assert.match(workflow, /schedule:[\s\S]*?workflow_dispatch:/);
  assert.match(workflow, /candidate-source:[\s\S]*?default: main/);
  assert.match(workflow, /pull-request:[\s\S]*?default: ""/);
  assert.match(workflow, /expected-head:[\s\S]*?default: ""/);
  assert.match(workflow, /HOSTED_CANDIDATE_SOURCE:[\s\S]*?inputs\.candidate-source \|\| 'main'/);
  assert.match(workflow, /HOSTED_PULL_REQUEST:[\s\S]*?inputs\.pull-request \|\| ''/);
  assert.match(workflow, /HOSTED_EXPECTED_HEAD:[\s\S]*?inputs\.expected-head \|\| ''/);
  assert.match(workflow, /matrix:[\s\S]*?variant: baseline[\s\S]*?variant: candidate/);
  assert.match(
    workflow,
    /needs\.resolve\.outputs\.baseline[\s\S]*?needs\.resolve\.outputs\.candidate/,
  );
});

test("rejects closed, forked, wrong-base, moved and malformed PR identities", () => {
  const pullEndpoint = `repos/${repository}/pulls/2585`;
  const commitEndpoint = `repos/${repository}/commits/${expectedHead}`;
  const pull = savedResponses[pullEndpoint];
  const commit = savedResponses[commitEndpoint];
  for (const badPull of [
    { ...pull, ...invalid.closed },
    { ...pull, head: { ...pull.head, ...invalid.forkHead.head } },
    { ...pull, base: { ...pull.base, ...invalid.wrongBase.base } },
    { ...pull, base: { ...pull.base, ...invalid.forkBase.base } },
    { ...pull, ...invalid.wrongNumber },
    { ...pull, ...invalid.wrongUrl },
    { ...pull, head: { ...pull.head, ...invalid.movedHead.head } },
  ]) {
    assert.throws(
      () => resolve(manualPullRequest, { [pullEndpoint]: badPull }),
      /open same-repository PR/,
    );
  }
  for (const badCommit of [
    { ...commit, ...invalid.movedCommit },
    { ...commit, ...invalid.invalidTree },
    invalid.missingTree,
  ]) {
    assert.throws(() => resolve(manualPullRequest, { [commitEndpoint]: badCommit }));
  }
});

test("provenance validation rejects PR metadata that no longer describes the selected repository", () => {
  const candidate = resolve(manualPullRequest).candidate;
  for (const patch of [
    { kind: invalid.wrongKind },
    { event: invalid.scheduledEvent },
    { ref: invalid.wrongCandidateRef },
    { commit: invalid.differentCandidateCommit },
    { pullRequest: { ...candidate.pullRequest, headRepository: invalid.forkRepository } },
    { pullRequest: { ...candidate.pullRequest, baseRef: invalid.wrongBaseRef } },
    {
      pullRequest: {
        ...candidate.pullRequest,
        expectedHeadCommit: invalid.differentExpectedHead,
      },
    },
  ]) {
    assert.throws(
      () => validateCandidateSource({ ...candidate, ...patch }, repository),
      /Candidate source must be main or an exact manual same-repository pull request/,
    );
  }
});
