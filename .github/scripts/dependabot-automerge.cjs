const { readFileSync } = require("node:fs");

const { fixtureOnlyDecision } = require("./dependabot-pr-policy.cjs");

const API_ROOT = "https://api.github.com";
const REQUEST_TIMEOUT_MS = 30_000;

const ENABLE_AUTO_MERGE_QUERY = `
  mutation($pullRequestId: ID!) {
    enablePullRequestAutoMerge(input: {
      pullRequestId: $pullRequestId,
      mergeMethod: SQUASH
    }) {
      pullRequest { autoMergeRequest { enabledAt } }
    }
  }
`;

function isMajorBump(from, to) {
  const major = (version) => parseInt(version.replace(/^v/, "").split(".")[0], 10);
  return major(to) > major(from);
}

function hasMajorBump(pullRequest) {
  const titleMatch = pullRequest.title.match(/from\s+v?([\d.]+)\s+to\s+v?([\d.]+)/i);
  if (titleMatch) {
    return isMajorBump(titleMatch[1], titleMatch[2]);
  }

  const body = pullRequest.body ?? "";
  const bodyMatches = [...body.matchAll(/from\s+`?v?([\d.]+)`?\s+to\s+`?v?([\d.]+)`?/gi)];
  return bodyMatches.some((match) => isMajorBump(match[1], match[2]));
}

function hasActiveApproval(reviews) {
  const latestStateByUser = new Map();
  const sortedReviews = [...reviews].sort((left, right) => {
    const leftTime = new Date(left.submitted_at ?? 0).getTime();
    const rightTime = new Date(right.submitted_at ?? 0).getTime();
    return leftTime - rightTime;
  });

  for (const review of sortedReviews) {
    if (!review.user?.login) {
      continue;
    }
    latestStateByUser.set(review.user.login, review.state);
  }

  return [...latestStateByUser.values()].includes("APPROVED");
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

function formatWorkflowWarning(message) {
  const escaped = String(message)
    .replace(/%/g, "%25")
    .replace(/\r/g, "%0D")
    .replace(/\n/g, "%0A");
  return `::warning::${escaped}`;
}

function nextLink(link) {
  if (!link) {
    return null;
  }
  for (const part of link.split(",")) {
    const match = part.match(/<([^>]+)>;\s*rel="next"/);
    if (match) {
      return match[1];
    }
  }
  return null;
}

function approvalWarning(error) {
  return (
    "Could not approve PR with GITHUB_TOKEN: "
    + `${errorMessage(error)}. If approvals are required, enable `
    + '"Allow GitHub Actions to create and approve pull requests" '
    + "in the repository Actions settings."
  );
}

function autoMergeWarning(error) {
  return (
    "Could not enable auto-merge: "
    + `${errorMessage(error)}. Confirm repository auto-merge is `
    + "enabled and squash merging is allowed."
  );
}

async function evaluateDependabotPullRequest({ pr, owner, repo, github, info, warning }) {
  info(`Evaluating PR #${pr.number} (${pr.title})`);

  const files = await github.paginateFiles(owner, repo, pr.number);
  const changedPaths = files.map((file) => file.filename);
  const fixtureDecision = fixtureOnlyDecision(changedPaths, pr.changed_files);

  if (fixtureDecision === "incomplete") {
    warning(
      `Skipping fixture-only policy because GitHub returned ${files.length} `
      + `of ${pr.changed_files} changed files`,
    );
    return;
  }

  if (fixtureDecision === "close") {
    info("Closing fixture-only Dependabot pull request");
    await github.updatePullRequest(owner, repo, pr.number, { state: "closed" });
    return;
  }

  if (pr.draft) {
    info("Skipping draft pull request");
    return;
  }

  if (hasMajorBump(pr)) {
    info("Skipping semver-major update");
    return;
  }

  const reviews = await github.listReviews(owner, repo, pr.number);
  if (hasActiveApproval(reviews)) {
    info("Approval already present");
  } else {
    info("Approving PR");
    try {
      await github.createReview(owner, repo, pr.number, { event: "APPROVE" });
    } catch (error) {
      warning(approvalWarning(error));
    }
  }

  const freshPr = await github.getPullRequest(owner, repo, pr.number);
  if (freshPr.auto_merge) {
    info("Auto-merge already enabled");
    return;
  }

  info("Enabling auto-merge");
  try {
    await github.graphql(ENABLE_AUTO_MERGE_QUERY, { pullRequestId: freshPr.node_id });
    info("Done: auto-merge enabled");
  } catch (error) {
    warning(autoMergeWarning(error));
  }
}

function createFetchClient(token, fetchImpl = globalThis.fetch) {
  async function request(url, { method = "GET", body } = {}) {
    const response = await fetchImpl(url, {
      method,
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "User-Agent": "dependabot-automerge",
        "X-GitHub-Api-Version": "2022-11-28",
        ...(body ? { "Content-Type": "application/json" } : {}),
      },
      body: body ? JSON.stringify(body) : undefined,
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    const text = await response.text();
    let payload = null;
    if (text) {
      try {
        payload = JSON.parse(text);
      } catch {
        payload = text;
      }
    }
    if (!response.ok) {
      const message = payload && typeof payload === "object" && payload.message
        ? payload.message
        : `${response.status} ${response.statusText}`;
      throw new Error(message);
    }
    return { data: payload, link: response.headers.get("link") };
  }

  return {
    async paginateFiles(owner, repo, pullNumber) {
      const files = [];
      let url = `${API_ROOT}/repos/${owner}/${repo}/pulls/${pullNumber}/files?per_page=100`;
      while (url) {
        const page = await request(url);
        files.push(...page.data);
        url = nextLink(page.link);
      }
      return files;
    },
    async updatePullRequest(owner, repo, pullNumber, body) {
      const page = await request(
        `${API_ROOT}/repos/${owner}/${repo}/pulls/${pullNumber}`,
        { method: "PATCH", body },
      );
      return page.data;
    },
    async listReviews(owner, repo, pullNumber) {
      const page = await request(
        `${API_ROOT}/repos/${owner}/${repo}/pulls/${pullNumber}/reviews?per_page=100`,
      );
      return page.data;
    },
    async createReview(owner, repo, pullNumber, body) {
      const page = await request(
        `${API_ROOT}/repos/${owner}/${repo}/pulls/${pullNumber}/reviews`,
        { method: "POST", body },
      );
      return page.data;
    },
    async getPullRequest(owner, repo, pullNumber) {
      const page = await request(`${API_ROOT}/repos/${owner}/${repo}/pulls/${pullNumber}`);
      return page.data;
    },
    async graphql(query, variables) {
      const page = await request(`${API_ROOT}/graphql`, {
        method: "POST",
        body: { query, variables },
      });
      if (!page.data || typeof page.data !== "object" || Array.isArray(page.data)) {
        throw new Error("GraphQL response was empty");
      }
      if (Array.isArray(page.data.errors) && page.data.errors.length > 0) {
        throw new Error(page.data.errors.map((error) => error.message).join("\n"));
      }
      return page.data.data;
    },
  };
}

async function main() {
  const token = process.env.GITHUB_TOKEN;
  if (!token) {
    throw new Error("GITHUB_TOKEN is required");
  }
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) {
    throw new Error("GITHUB_EVENT_PATH is required");
  }
  const repository = process.env.GITHUB_REPOSITORY;
  const separator = repository ? repository.indexOf("/") : -1;
  if (separator <= 0 || separator === repository.length - 1) {
    throw new Error("GITHUB_REPOSITORY is required");
  }
  const event = JSON.parse(readFileSync(eventPath, "utf8"));
  await evaluateDependabotPullRequest({
    pr: event.pull_request,
    owner: repository.slice(0, separator),
    repo: repository.slice(separator + 1),
    github: createFetchClient(token),
    info: (message) => {
      process.stdout.write(`${message}\n`);
    },
    warning: (message) => {
      process.stdout.write(`${formatWorkflowWarning(message)}\n`);
    },
  });
}

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${errorMessage(error)}\n`);
    process.exitCode = 1;
  });
}

module.exports = {
  ENABLE_AUTO_MERGE_QUERY,
  approvalWarning,
  autoMergeWarning,
  createFetchClient,
  evaluateDependabotPullRequest,
  formatWorkflowWarning,
  hasActiveApproval,
  hasMajorBump,
};
