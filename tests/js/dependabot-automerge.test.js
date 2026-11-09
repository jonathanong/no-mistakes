const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const {
  ENABLE_AUTO_MERGE_QUERY,
  approvalWarning,
  autoMergeWarning,
  createFetchClient,
  evaluateDependabotPullRequest,
  formatWorkflowWarning,
  hasActiveApproval,
  hasMajorBump,
} = require(join(__dirname, "..", "..", ".github", "scripts", "dependabot-automerge.cjs"));

function pullRequest(overrides = {}) {
  return {
    number: 42,
    title: "bump left-pad from 1.2.3 to 1.2.4",
    body: "",
    draft: false,
    changed_files: 1,
    node_id: "PR_node",
    ...overrides,
  };
}

function recordingGithub(files, extras = {}) {
  const calls = [];
  return {
    calls,
    async paginateFiles() {
      calls.push("files");
      return files;
    },
    async updatePullRequest(_owner, _repo, _pullNumber, body) {
      calls.push(["update", body]);
    },
    async listReviews() {
      calls.push("reviews");
      return extras.reviews ?? [];
    },
    async createReview(_owner, _repo, _pullNumber, body) {
      calls.push(["review", body]);
      if (extras.reviewError) {
        throw extras.reviewError;
      }
    },
    async getPullRequest() {
      calls.push("get");
      return extras.freshPr ?? { node_id: "PR_node", auto_merge: null };
    },
    async graphql(_query, variables) {
      calls.push(["graphql", variables, _query]);
      if (extras.graphqlError) {
        throw extras.graphqlError;
      }
    },
  };
}

async function run(pr, files, extras) {
  const github = recordingGithub(files, extras);
  const info = [];
  const warning = [];
  await evaluateDependabotPullRequest({
    pr,
    owner: "octo",
    repo: "repo",
    github,
    info: (message) => info.push(message),
    warning: (message) => warning.push(message),
  });
  return { calls: github.calls, info, warning };
}

test("closes a fixture-only Dependabot pull request", async () => {
  const result = await run(
    pullRequest({ draft: true, title: "bump fixture from 1.0.0 to 2.0.0", changed_files: 2 }),
    [
      { filename: "fixtures/example/package.json" },
      { filename: "test-cases/example/Cargo.toml" },
    ],
  );

  assert.deepEqual(result.calls, ["files", ["update", { state: "closed" }]]);
  assert.deepEqual(result.info, [
    "Evaluating PR #42 (bump fixture from 1.0.0 to 2.0.0)",
    "Closing fixture-only Dependabot pull request",
  ]);
  assert.deepEqual(result.warning, []);
});

test("warns and stops when GitHub truncates the changed-file list", async () => {
  const result = await run(
    pullRequest({ changed_files: 2 }),
    [{ filename: "fixtures/example/package.json" }],
  );

  assert.deepEqual(result.calls, ["files"]);
  assert.deepEqual(result.warning, [
    "Skipping fixture-only policy because GitHub returned 1 of 2 changed files",
  ]);
});

test("skips a draft pull request before the semver-major check", async () => {
  const result = await run(
    pullRequest({ draft: true, title: "bump left-pad from 1.2.3 to 2.0.0" }),
    [{ filename: "package.json" }],
  );

  assert.deepEqual(result.calls, ["files"]);
  assert.deepEqual(result.info.at(-1), "Skipping draft pull request");
});

test("skips a semver-major update from the title or body", async () => {
  const fromTitle = await run(
    pullRequest({ title: "bump actions/checkout from v1.2.3 to v2.0.0" }),
    [{ filename: "package.json" }],
  );
  const fromBody = await run(
    pullRequest({
      title: "bump left-pad",
      body: "Updates `left-pad` from `1.2.3` to `2.0.0`.",
    }),
    [{ filename: "package.json" }],
  );
  const titleWins = await run(
    pullRequest({
      title: "bump left-pad from 1.2.3 to 1.3.0",
      body: "from 1.0.0 to 2.0.0",
    }),
    [{ filename: "package.json" }],
  );

  assert.equal(fromTitle.info.at(-1), "Skipping semver-major update");
  assert.equal(fromBody.info.at(-1), "Skipping semver-major update");
  assert.deepEqual(fromTitle.calls, ["files"]);
  assert.equal(titleWins.calls.includes("reviews"), true);
  assert.equal(hasMajorBump(pullRequest({ title: "bump left-pad from 1.2.3 to 1.3.0" })), false);
});

test("does not treat a deleted review author as an approval", async () => {
  const reviews = [
    { id: 7, user: null, state: "APPROVED", submitted_at: "2026-01-01T00:00:00Z" },
  ];

  assert.equal(hasActiveApproval(reviews), false);
  assert.equal(hasActiveApproval([
    ...reviews,
    { id: 8, user: { login: "alice" }, state: "APPROVED", submitted_at: "2026-01-02T00:00:00Z" },
  ]), true);
  const result = await run(pullRequest(), [{ filename: "package.json" }], { reviews });

  assert.equal(result.info.includes("Approving PR"), true);
  assert.equal(result.calls.some((call) => Array.isArray(call) && call[0] === "review"), true);
});

test("approves and enables squash auto-merge", async () => {
  const result = await run(pullRequest(), [{ filename: "package.json" }]);

  assert.deepEqual(result.calls, [
    "files",
    "reviews",
    ["review", { event: "APPROVE" }],
    "get",
    ["graphql", { pullRequestId: "PR_node" }, ENABLE_AUTO_MERGE_QUERY],
  ]);
  assert.match(ENABLE_AUTO_MERGE_QUERY, /mergeMethod:\s*SQUASH/);
  assert.deepEqual(result.info, [
    "Evaluating PR #42 (bump left-pad from 1.2.3 to 1.2.4)",
    "Approving PR",
    "Enabling auto-merge",
    "Done: auto-merge enabled",
  ]);
  assert.deepEqual(result.warning, []);
});

test("keeps an existing approval and still enables auto-merge", async () => {
  const result = await run(pullRequest(), [{ filename: "package.json" }], {
    reviews: [
      { user: { login: "alice" }, state: "APPROVED", submitted_at: "2026-01-01T00:00:00Z" },
      { user: { login: "bob" }, state: "CHANGES_REQUESTED", submitted_at: "2026-01-02T00:00:00Z" },
      { user: { login: "bob" }, state: "APPROVED", submitted_at: "2026-01-03T00:00:00Z" },
    ],
  });

  assert.equal(result.calls.some((call) => Array.isArray(call) && call[0] === "review"), false);
  assert.equal(result.info.includes("Approval already present"), true);
  assert.equal(result.calls.at(-1)[0], "graphql");
  assert.equal(hasActiveApproval([
    { user: { login: "alice" }, state: "APPROVED", submitted_at: "2026-01-02T00:00:00Z" },
    { user: { login: "alice" }, state: "DISMISSED", submitted_at: "2026-01-03T00:00:00Z" },
  ]), false);
});

test("warns when approval fails and still tries auto-merge", async () => {
  const result = await run(pullRequest(), [{ filename: "package.json" }], {
    reviewError: new Error("Resource not accessible by integration"),
  });

  assert.deepEqual(result.warning, [
    approvalWarning(new Error("Resource not accessible by integration")),
  ]);
  assert.equal(result.info.includes("Enabling auto-merge"), true);
  assert.equal(result.info.at(-1), "Done: auto-merge enabled");
});

test("skips enabling auto-merge when it is already enabled", async () => {
  const result = await run(pullRequest(), [{ filename: "package.json" }], {
    freshPr: { node_id: "PR_node", auto_merge: { enabled_by: { login: "dependabot[bot]" } } },
  });

  assert.equal(result.calls.some((call) => Array.isArray(call) && call[0] === "graphql"), false);
  assert.equal(result.info.at(-1), "Auto-merge already enabled");
});

test("warns when squash auto-merge cannot be enabled", async () => {
  const result = await run(pullRequest(), [{ filename: "package.json" }], {
    graphqlError: "squash merges are disabled",
  });

  assert.deepEqual(result.warning, [autoMergeWarning("squash merges are disabled")]);
  assert.equal(result.info.at(-1), "Enabling auto-merge");
});

test("loads files, reviews, and the squash mutation through native fetch", async () => {
  const seen = [];
  const fetchImpl = async (url, options) => {
    seen.push({ url: String(url), options });
    const address = String(url);
    if (address.endsWith("/graphql")) {
      return new Response(JSON.stringify({
        data: { enablePullRequestAutoMerge: { pullRequest: { autoMergeRequest: { enabledAt: "now" } } } },
      }), { status: 200 });
    }
    if (address.includes("/files")) {
      const page = address.includes("page=2")
        ? [{ filename: "package.json" }]
        : [{ filename: "fixtures/example/package.json" }];
      const headers = address.includes("page=2")
        ? {}
        : { link: `<https://api.github.com/repos/octo/repo/pulls/42/files?per_page=100&page=2>; rel="next"` };
      return new Response(JSON.stringify(page), { status: 200, headers });
    }
    if (address.includes("/reviews") && options.method === "POST") {
      return new Response(JSON.stringify({ id: 1 }), { status: 201 });
    }
    if (address.includes("/reviews")) {
      return new Response(JSON.stringify([]), { status: 200 });
    }
    if (options.method === "PATCH") {
      return new Response(JSON.stringify({ state: "closed" }), { status: 200 });
    }
    return new Response(JSON.stringify({ node_id: "PR_node", auto_merge: null }), { status: 200 });
  };

  const github = createFetchClient("token-value", fetchImpl);
  const result = await runWith(github, pullRequest({ changed_files: 2 }));

  assert.equal(seen[0].options.headers.Authorization, "Bearer token-value");
  assert.equal(seen[0].options.headers.Accept, "application/vnd.github+json");
  assert.equal(seen.filter((call) => call.url.includes("/files")).length, 2);
  const graphql = seen.find((call) => call.url.endsWith("/graphql"));
  const body = JSON.parse(graphql.options.body);
  assert.equal(body.variables.pullRequestId, "PR_node");
  assert.match(body.query, /mergeMethod:\s*SQUASH/);
  assert.equal(result.info.at(-1), "Done: auto-merge enabled");
});

test("surfaces REST and GraphQL failures from native fetch", async () => {
  const github = createFetchClient("token-value", async (url, options = {}) => {
    const address = String(url);
    if (address.endsWith("/graphql")) {
      return new Response(JSON.stringify({
        data: null,
        errors: [{ message: "Auto merge is not allowed" }],
      }), { status: 200 });
    }
    if (address.includes("/reviews")) {
      if (options.method === "POST") {
        return new Response(JSON.stringify({ message: "Resource not accessible by integration" }), {
          status: 403,
          statusText: "Forbidden",
        });
      }
      return new Response(JSON.stringify([]), { status: 200 });
    }
    if (address.includes("/files")) {
      return new Response(JSON.stringify([{ filename: "package.json" }]), { status: 200 });
    }
    return new Response(JSON.stringify({ node_id: "PR_node", auto_merge: null }), { status: 200 });
  });
  const warning = [];
  const info = [];
  await evaluateDependabotPullRequest({
    pr: pullRequest(),
    owner: "octo",
    repo: "repo",
    github,
    info: (message) => info.push(message),
    warning: (message) => warning.push(message),
  });

  assert.deepEqual(warning, [
    approvalWarning(new Error("Resource not accessible by integration")),
    autoMergeWarning(new Error("Auto merge is not allowed")),
  ]);
});

test("keeps the auto-merge workflow free of third-party actions", () => {
  const workflow = readFileSync(
    join(__dirname, "..", "..", ".github", "workflows", "dependabot-pr-automerge.yml"),
    "utf8",
  );

  assert.equal(workflow.includes("uses:"), false);
  assert.match(workflow, /^on:\n {2}pull_request:\n/m);
  assert.match(workflow, /github\.event\.pull_request\.base\.sha/);
  assert.match(workflow, /dependabot-automerge\.cjs/);
  assert.match(workflow, /dependabot-pr-policy\.cjs/);
  assert.match(workflow, /gh api/);
  assert.equal(formatWorkflowWarning("line\npercent %"), "::warning::line%0Apercent %25");
});

async function runWith(github, pr) {
  const info = [];
  const warning = [];
  await evaluateDependabotPullRequest({
    pr,
    owner: "octo",
    repo: "repo",
    github,
    info: (message) => info.push(message),
    warning: (message) => warning.push(message),
  });
  return { info, warning };
}
