const fs = require("node:fs");

function runId(value) {
  if (!/^[1-9]\d{0,15}$/.test(value || "") || !Number.isSafeInteger(Number(value)))
    throw new Error("Build source run ID must be a positive safe integer");
  return value;
}

async function readSource(url, token, fetchRun, kind) {
  try {
    const response = await fetchRun(url, {
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "X-GitHub-Api-Version": "2022-11-28",
      },
      signal: AbortSignal.timeout(15000),
      redirect: "error",
    });
    if (!response.ok) throw new Error("Unsuccessful response");
    return await response.json();
  } catch {
    throw new Error(`Unable to read the build source ${kind}`);
  }
}

async function resolveSource(env, fetchRun = fetch) {
  const cold = env.COLD_BUILD || "false";
  if (!["true", "false"].includes(cold)) throw new Error("COLD_BUILD must be true or false");
  let reuse = env.REUSE_BUILD_RUN_ID || "";
  const prefix = "refs/heads/ci/warm-build-";
  const warmPush = env.GITHUB_EVENT_NAME === "push" && env.GITHUB_REF?.startsWith(prefix);
  if (warmPush) {
    const inferred = runId(env.GITHUB_REF.slice(prefix.length));
    if (reuse && reuse !== inferred)
      throw new Error("Build source input must match the warm-build branch run ID");
    reuse = inferred;
  }
  if (!reuse) return { run_id: runId(env.GITHUB_RUN_ID), reuse: false };
  if (env.GITHUB_EVENT_NAME !== "workflow_dispatch" && !warmPush)
    throw new Error("Build artifact reuse requires workflow_dispatch or a warm-build push branch");
  if (cold === "true") throw new Error("Cold builds cannot reuse build artifacts");
  const id = runId(reuse);
  const repo = env.GITHUB_REPOSITORY;
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repo || ""))
    throw new Error("Invalid GitHub repository");
  if (!/^[a-f0-9]{40}$/.test(env.GITHUB_SHA || "")) throw new Error("Invalid GitHub commit SHA");
  if (!env.GITHUB_TOKEN) throw new Error("Build artifact reuse requires a read token");
  const url = `https://api.github.com/repos/${repo}/actions/runs/${id}`;
  const source = await readSource(url, env.GITHUB_TOKEN, fetchRun, "workflow run");
  const measurement =
    source?.event === "workflow_dispatch" ||
    (source?.event === "push" && source.head_branch === "ci/cold-build");
  if (
    source?.status !== "completed" ||
    source.conclusion !== "success" ||
    !measurement ||
    source.path !== ".github/workflows/ci.yml" ||
    source.head_sha !== env.GITHUB_SHA ||
    source.head_repository?.full_name !== repo
  )
    throw new Error(
      "Build source must be a successful completed CI measurement for this repository and exact commit",
    );
  const inventory = await readSource(
    `${url}/artifacts?per_page=100`,
    env.GITHUB_TOKEN,
    fetchRun,
    "artifact inventory",
  );
  const required = [
    "linux-runtime-cli",
    "linux-runtime-addon",
    "native-runtime-x86_64-pc-windows-msvc",
    "native-runtime-aarch64-apple-darwin",
    "native-rust-archive-x86_64-pc-windows-msvc",
    "native-rust-archive-aarch64-apple-darwin",
    "standalone-library",
    "rust-coverage-build",
  ];
  if (
    !Number.isInteger(inventory?.total_count) ||
    inventory.total_count < required.length ||
    inventory.total_count > 100 ||
    !Array.isArray(inventory.artifacts) ||
    inventory.artifacts.length !== inventory.total_count ||
    inventory.artifacts.some(
      (artifact) =>
        !artifact || typeof artifact.name !== "string" || typeof artifact.expired !== "boolean",
    ) ||
    required.some((name) => {
      const matches = inventory.artifacts.filter((artifact) => artifact?.name === name);
      return (
        matches.length !== 1 ||
        matches[0].expired !== false ||
        typeof matches[0].expires_at !== "string" ||
        !(Date.parse(matches[0].expires_at) > Date.now())
      );
    })
  )
    throw new Error(
      "Build source must contain a complete inventory of at most 100 artifacts and all eight unexpired build archives",
    );
  return { run_id: id, reuse: true };
}

async function main(env = process.env) {
  const source = await resolveSource(env);
  if (!env.GITHUB_OUTPUT) throw new Error("GITHUB_OUTPUT is required");
  fs.appendFileSync(env.GITHUB_OUTPUT, `run_id=${source.run_id}\nreuse=${source.reuse}\n`);
}

if (require.main === module) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}

module.exports = { resolveSource };
