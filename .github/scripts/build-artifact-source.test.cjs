const assert = require("node:assert/strict");
const { test } = require("node:test");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { resolveSource } = require("./build-artifact-source.cjs");
const saved = require("../../test-cases/build-artifact-source/scenarios.json");

function unexpectedFetch() {
  assert.fail("invalid or default inputs must not request a workflow run");
}

test("default and cold builds select the current run without network access", async () => {
  for (const cold of ["", "false", "true"]) {
    assert.deepEqual(
      await resolveSource(
        { ...saved.environment, REUSE_BUILD_RUN_ID: "", COLD_BUILD: cold },
        unexpectedFetch,
      ),
      {
        run_id: saved.environment.GITHUB_RUN_ID,
        reuse: false,
      },
    );
  }
});

test("warm dispatch validates the fixed API source and exact commit", async () => {
  let calls = 0;
  const source = await resolveSource(saved.environment, async (url, options) => {
    calls++;
    assert.equal(
      url,
      "https://api.github.com/repos/jonathanong/no-mistakes/actions/runs/43404256731",
    );
    assert.equal(options.headers.Authorization, `Bearer ${saved.environment.GITHUB_TOKEN}`);
    assert.equal(options.redirect, "error");
    assert.ok(options.signal instanceof AbortSignal);
    return { ok: true, json: async () => saved.run };
  });
  assert.deepEqual(source, { run_id: saved.environment.REUSE_BUILD_RUN_ID, reuse: true });
  assert.equal(calls, 1);
});

test("invalid reuse provenance fails closed", async () => {
  for (const changes of saved.invalidRuns) {
    await assert.rejects(
      resolveSource(saved.environment, async () => ({
        ok: true,
        json: async () => ({ ...saved.run, ...changes }),
      })),
      /successful completed CI measurement/,
    );
  }
  await assert.rejects(
    resolveSource(saved.environment, async () => ({ ok: true, json: async () => null })),
    /successful completed CI measurement/,
  );
});

test("warm push infers the run ID and accepts only exact cold-push or dispatch provenance", async () => {
  const env = { ...saved.environment, ...saved.warmEnvironment };
  for (const changes of [{}, saved.coldRun]) {
    for (const explicit of ["", saved.environment.REUSE_BUILD_RUN_ID]) {
      assert.deepEqual(
        await resolveSource({ ...env, REUSE_BUILD_RUN_ID: explicit }, async (url) => {
          assert.ok(url.endsWith(`/actions/runs/${saved.environment.REUSE_BUILD_RUN_ID}`));
          return { ok: true, json: async () => ({ ...saved.run, ...changes }) };
        }),
        { run_id: saved.environment.REUSE_BUILD_RUN_ID, reuse: true },
      );
    }
  }
  for (const branch of saved.invalidSourceBranches) {
    await assert.rejects(
      resolveSource(env, async () => ({
        ok: true,
        json: async () => ({ ...saved.run, ...saved.coldRun, head_branch: branch }),
      })),
      /successful completed CI measurement/,
    );
  }
  for (const changes of saved.invalidRuns) {
    await assert.rejects(
      resolveSource(env, async () => ({
        ok: true,
        json: async () => ({ ...saved.run, ...saved.coldRun, ...changes }),
      })),
      /successful completed CI measurement/,
    );
  }
});

test("measurement branch parsing rejects malformed IDs, explicit mismatches and cold reuse", async () => {
  const env = { ...saved.environment, ...saved.warmEnvironment };
  for (const ref of saved.invalidWarmRefs) {
    await assert.rejects(
      resolveSource({ ...env, GITHUB_REF: ref }, unexpectedFetch),
      /positive safe integer/,
    );
  }
  await assert.rejects(
    resolveSource({ ...env, REUSE_BUILD_RUN_ID: saved.environment.GITHUB_RUN_ID }, unexpectedFetch),
    /must match/,
  );
  await assert.rejects(
    resolveSource({ ...env, COLD_BUILD: "true" }, unexpectedFetch),
    /cannot reuse/,
  );
  await assert.rejects(
    resolveSource(
      {
        ...env,
        GITHUB_EVENT_NAME: "pull_request",
        REUSE_BUILD_RUN_ID: saved.environment.REUSE_BUILD_RUN_ID,
      },
      unexpectedFetch,
    ),
    /requires/,
  );
  assert.deepEqual(
    await resolveSource(
      { ...env, GITHUB_REF: "refs/heads/ci/cold-build", COLD_BUILD: "true" },
      unexpectedFetch,
    ),
    {
      run_id: saved.environment.GITHUB_RUN_ID,
      reuse: false,
    },
  );
});

test("invalid source IDs and dispatch settings fail before API access", async () => {
  for (const id of saved.invalidIds) {
    await assert.rejects(
      resolveSource({ ...saved.environment, REUSE_BUILD_RUN_ID: id }, unexpectedFetch),
      /positive safe integer/,
    );
    await assert.rejects(
      resolveSource(
        { ...saved.environment, REUSE_BUILD_RUN_ID: "", GITHUB_RUN_ID: id },
        unexpectedFetch,
      ),
      /positive safe integer/,
    );
  }
  for (const changes of saved.invalidEnvironments) {
    await assert.rejects(resolveSource({ ...saved.environment, ...changes }, unexpectedFetch));
  }
});

test("API failures do not expose response bodies, tokens, or underlying errors", async () => {
  for (const fetchRun of [
    async () => ({ ok: false }),
    async () => {
      throw new Error(saved.environment.GITHUB_TOKEN);
    },
    async () => ({
      ok: true,
      json: async () => {
        throw new Error(saved.environment.GITHUB_TOKEN);
      },
    }),
  ])
    await assert.rejects(resolveSource(saved.environment, fetchRun), {
      message: "Unable to read the build source workflow run",
    });
});

test("CLI writes only validated source outputs and fails without partial outputs", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "build-artifact-source-"));
  const output = path.join(directory, "github-output");
  const env = {
    ...process.env,
    ...saved.environment,
    REUSE_BUILD_RUN_ID: "",
    GITHUB_OUTPUT: output,
  };
  try {
    const success = spawnSync(process.execPath, [require.resolve("./build-artifact-source.cjs")], {
      env,
      encoding: "utf8",
    });
    assert.equal(success.status, 0, success.stderr);
    const expected = `run_id=${saved.environment.GITHUB_RUN_ID}\nreuse=false\n`;
    assert.equal(fs.readFileSync(output, "utf8"), expected);
    assert.equal(success.stdout, "");
    const failure = spawnSync(process.execPath, [require.resolve("./build-artifact-source.cjs")], {
      env: { ...env, GITHUB_RUN_ID: "0" },
      encoding: "utf8",
    });
    assert.equal(failure.status, 1);
    assert.equal(fs.readFileSync(output, "utf8"), expected);
    assert.ok(!failure.stderr.includes(saved.environment.GITHUB_TOKEN));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
