const COMMENT_MARKER_PREFIX = "<!-- native-job-timing:";

function commentMarker(jobName) {
  return `${COMMENT_MARKER_PREFIX} ${jobName} -->`;
}

function parseTimestamp(value) {
  if (!value) {
    return null;
  }
  const ms = Date.parse(value);
  return Number.isFinite(ms) ? ms : null;
}

function durationSeconds(startedAt, completedAt, nowMs = Date.now()) {
  const start = parseTimestamp(startedAt);
  if (start == null) {
    return null;
  }
  const end = parseTimestamp(completedAt) ?? nowMs;
  return Math.max(0, Math.round((end - start) / 1000));
}

function formatDuration(seconds) {
  if (seconds == null) {
    return "n/a";
  }
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  if (minutes === 0) {
    return `${rest}s`;
  }
  return `${minutes}m ${rest}s`;
}

function formatDelta(afterSeconds, beforeSeconds) {
  if (afterSeconds == null || beforeSeconds == null) {
    return "n/a";
  }
  const delta = afterSeconds - beforeSeconds;
  const sign = delta > 0 ? "+" : "";
  return `${sign}${formatDuration(Math.abs(delta))}${delta < 0 ? " faster" : delta > 0 ? " slower" : ""}`;
}

function interestingStep(name) {
  return (
    name === "Run platform-specific Rust tests" ||
    name === "Build native CLI and N-API addon" ||
    name.startsWith("Exclude workspace") ||
    name.startsWith("Disable Microsoft Defender")
  );
}

function stepDurationMap(job, nowMs) {
  const map = new Map();
  for (const step of job.steps ?? []) {
    if (!interestingStep(step.name)) {
      continue;
    }
    map.set(step.name, durationSeconds(step.started_at, step.completed_at, nowMs));
  }
  return map;
}

function buildMarkdown({ jobName, afterJob, beforeJob, afterSha, beforeSha, nowMs = Date.now() }) {
  const afterSeconds = durationSeconds(afterJob.started_at, afterJob.completed_at, nowMs);
  const beforeSeconds = beforeJob
    ? durationSeconds(beforeJob.started_at, beforeJob.completed_at, nowMs)
    : null;
  const afterSteps = stepDurationMap(afterJob, nowMs);
  const beforeSteps = beforeJob ? stepDurationMap(beforeJob, nowMs) : new Map();
  const stepNames = [...new Set([...beforeSteps.keys(), ...afterSteps.keys()])];

  const lines = [
    commentMarker(jobName),
    `## Native job timing (${jobName})`,
    "",
    "| | Duration | SHA |",
    "| --- | --- | --- |",
    `| Before (base) | ${formatDuration(beforeSeconds)} | ${beforeSha ? `\`${beforeSha.slice(0, 7)}\`` : "n/a"} |`,
    `| After (this run) | ${formatDuration(afterSeconds)} | ${afterSha ? `\`${afterSha.slice(0, 7)}\`` : "n/a"} |`,
    `| Delta | **${formatDelta(afterSeconds, beforeSeconds)}** | |`,
    "",
  ];

  if (stepNames.length > 0) {
    lines.push("| Step | Before | After |");
    lines.push("| --- | --- | --- |");
    for (const name of stepNames) {
      lines.push(
        `| ${name} | ${formatDuration(beforeSteps.get(name) ?? null)} | ${formatDuration(afterSteps.get(name) ?? null)} |`,
      );
    }
    lines.push("");
  }

  if (!beforeJob) {
    lines.push(
      "_No completed native job found on the PR base SHA or base branch to compare against._",
    );
    lines.push("");
  }

  return lines.join("\n");
}

async function ghJson(args) {
  const { execFileSync } = require("node:child_process");
  const output = execFileSync("gh", args, {
    encoding: "utf8",
    env: process.env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  return JSON.parse(output);
}

function findJob(jobs, jobName) {
  return (jobs ?? []).find((job) => job.name === jobName) ?? null;
}

async function loadJobs(repository, runId) {
  const payload = await ghJson([
    "api",
    `repos/${repository}/actions/runs/${runId}/jobs?per_page=100`,
  ]);
  return payload.jobs ?? [];
}

async function findBeforeJob({ repository, workflow, jobName, baseSha, baseRef }) {
  const shaRuns = await ghJson([
    "run",
    "list",
    "--repo",
    repository,
    "--workflow",
    workflow,
    "--commit",
    baseSha,
    "--json",
    "databaseId,conclusion,status,headSha",
    "--limit",
    "20",
  ]);
  for (const run of shaRuns) {
    if (run.status !== "completed") {
      continue;
    }
    const jobs = await loadJobs(repository, run.databaseId);
    const job = findJob(jobs, jobName);
    if (job) {
      return { job, sha: run.headSha ?? baseSha };
    }
  }

  const branchRuns = await ghJson([
    "run",
    "list",
    "--repo",
    repository,
    "--workflow",
    workflow,
    "--branch",
    baseRef,
    "--status",
    "success",
    "--json",
    "databaseId,headSha",
    "--limit",
    "10",
  ]);
  for (const run of branchRuns) {
    const jobs = await loadJobs(repository, run.databaseId);
    const job = findJob(jobs, jobName);
    if (job) {
      return { job, sha: run.headSha };
    }
  }
  return { job: null, sha: null };
}

function ghWrite(args, payload) {
  const { spawnSync } = require("node:child_process");
  const result = spawnSync("gh", args, {
    input: payload,
    encoding: "utf8",
    env: process.env,
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || `gh ${args.join(" ")} failed`);
  }
}

async function upsertComment({ repository, prNumber, marker, body }) {
  const comments = await ghJson([
    "api",
    `repos/${repository}/issues/${prNumber}/comments?per_page=100`,
  ]);
  const existing = (comments ?? []).find((comment) => (comment.body ?? "").includes(marker));
  const payload = JSON.stringify({ body });
  if (existing) {
    ghWrite(
      [
        "api",
        "-X",
        "PATCH",
        `repos/${repository}/issues/comments/${existing.id}`,
        "-H",
        "Content-Type: application/json",
        "--input",
        "-",
      ],
      payload,
    );
    return;
  }

  ghWrite(
    [
      "api",
      "-X",
      "POST",
      `repos/${repository}/issues/${prNumber}/comments`,
      "-H",
      "Content-Type: application/json",
      "--input",
      "-",
    ],
    payload,
  );
}

async function main() {
  const repository = process.env.GITHUB_REPOSITORY;
  const runId = process.env.GITHUB_RUN_ID;
  const prNumber = process.env.PR_NUMBER;
  const jobName = process.env.JOB_NAME;
  const baseSha = process.env.BASE_SHA;
  const baseRef = process.env.BASE_REF;
  const workflow = process.env.WORKFLOW_NAME || "Test CI";
  const headSha = process.env.HEAD_SHA;

  if (!repository || !runId || !prNumber || !jobName || !baseSha || !baseRef) {
    throw new Error(
      "missing GITHUB_REPOSITORY, GITHUB_RUN_ID, PR_NUMBER, JOB_NAME, BASE_SHA, or BASE_REF",
    );
  }

  const afterJobs = await loadJobs(repository, runId);
  const afterJob = findJob(afterJobs, jobName);
  if (!afterJob) {
    throw new Error(`current run has no job named ${jobName}`);
  }

  const before = await findBeforeJob({
    repository,
    workflow,
    jobName,
    baseSha,
    baseRef,
  });
  const markdown = buildMarkdown({
    jobName,
    afterJob,
    beforeJob: before.job,
    afterSha: headSha,
    beforeSha: before.sha ?? baseSha,
  });

  const summaryPath = process.env.GITHUB_STEP_SUMMARY;
  if (summaryPath) {
    require("node:fs").appendFileSync(summaryPath, `${markdown}\n`);
  }

  await upsertComment({
    repository,
    prNumber,
    marker: commentMarker(jobName),
    body: markdown,
  });
  process.stdout.write(markdown);
}

module.exports = {
  buildMarkdown,
  commentMarker,
  durationSeconds,
  formatDelta,
  formatDuration,
  interestingStep,
};

if (require.main === module) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  });
}
