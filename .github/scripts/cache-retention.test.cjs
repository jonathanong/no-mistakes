const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const {
  plan,
  collect,
  retain,
  trustedEvent,
  summary,
  failureSummary,
} = require("./cache-retention.cjs");
const root = path.resolve(__dirname, "../..");
const fixture = JSON.parse(
  readFileSync(path.join(root, "test-cases/cache-retention/scenarios.json"), "utf8"),
);
const audit = JSON.parse(
  readFileSync(path.join(root, "test-cases/cache-retention/audit.json"), "utf8"),
);

test("closed PR records and older main versions are pruned while open PR and unknown caches survive", () => {
  const result = plan(fixture.versions, fixture.states, 400);
  assert.deepEqual(
    result.remove.map((cache) => cache.id),
    [1, 4],
  );
  assert.deepEqual(
    result.kept.map((cache) => cache.id),
    [2],
  );
  assert.equal(result.after, 300);
  assert.equal(result.withinBudget, true);
  assert.deepEqual(
    plan([...fixture.versions].reverse(), fixture.states, 400).remove,
    result.remove,
  );
  assert.deepEqual(
    plan(
      fixture.versions.filter((cache) => !result.remove.some((item) => item.id === cache.id)),
      fixture.states,
      undefined,
      fixture.sourceRunStates,
    ).remove,
    [],
  );
});

test("compatible shared dependencies retire legacy copies without merging compiler or RUSTFLAGS variants", () => {
  const result = plan(fixture.dependencies, {});
  assert.deepEqual(
    result.remove.map((cache) => cache.id),
    [12],
  );
  assert.deepEqual(
    result.kept.map((cache) => cache.id).sort((a, b) => a - b),
    [11, 13, 14, 15],
  );
  const withoutCommon = fixture.dependencies.filter((cache) => cache.id !== 11);
  assert.deepEqual(plan(withoutCommon, {}).remove, []);
});

test("real 35-entry audit can reach the 8 GiB snapshot budget while preserving all open PR records", () => {
  assert.equal(audit.total_count, 35);
  const result = plan(audit.actions_caches, fixture.states);
  assert.ok(result.withinBudget, summary(result));
  assert.equal(result.remove.filter((cache) => cache.ref === "refs/pull/3329/merge").length, 5);
  assert.equal(result.remove.filter((cache) => cache.ref === "refs/pull/3328/merge").length, 0);
  assert.ok(result.before > 10 * 1024 ** 3);
  const release = audit.actions_caches.find((cache) =>
    cache.key.startsWith("v0-rust-rust-quality-release-check-v1-"),
  );
  assert.ok(release);
  assert.equal(
    result.remove.some((cache) => cache.id === release.id),
    false,
  );
  const pnpm = audit.actions_caches.filter((cache) => cache.key.startsWith("pnpm-"));
  assert.equal(
    result.remove.some((cache) => pnpm.some((item) => item.id === cache.id)),
    false,
  );
  assert.match(summary(result), /New saves after the listing are not included/);
});

test("unprunable current families report excess instead of deleting protected records", () => {
  const result = plan(fixture.versions, fixture.states, 1);
  assert.equal(result.withinBudget, false);
  assert.match(summary(result), /protected/);
});

test("incomplete and duplicate paginated inventories fail before deletions", async () => {
  for (const total_count of [1001, 6])
    await assert.rejects(
      collect(async () => ({ total_count, actions_caches: fixture.versions })),
      /incomplete/,
    );
  const duplicatePage = Array.from({ length: 100 }, (_, index) => ({
    ...fixture.versions[0],
    id: index + 1,
  }));
  let calls = 0;
  await assert.rejects(
    collect(async () => {
      calls++;
      return { total_count: 200, actions_caches: duplicatePage };
    }),
    /changed while paging/,
  );
  assert.equal(calls, 2);
});

test("complete multi-page listings read each page once and dry-run never deletes", async () => {
  const caches = Array.from({ length: 101 }, (_, index) => ({
    ...fixture.versions[0],
    id: index + 1,
  }));
  const pages = [];
  const result = await collect(async (_, route) => {
    pages.push(route);
    return {
      total_count: 101,
      actions_caches: pages.length === 1 ? caches.slice(0, 100) : caches.slice(100),
    };
  });
  assert.equal(result.length, 101);
  assert.equal(pages.length, 2);
  const requests = [];
  await retain(async (method, route) => {
    requests.push([method, route]);
    if (route.startsWith("/actions/caches?"))
      return { total_count: fixture.versions.length, actions_caches: fixture.versions };
    return { state: fixture.states[route.split("/").pop()] };
  });
  assert.equal(requests.filter(([method]) => method === "DELETE").length, 0);
});

test("apply deletes only planned IDs after resolving all referenced pull requests", async () => {
  const requests = [];
  const result = await retain(async (method, route) => {
    requests.push([method, route]);
    if (route.startsWith("/actions/caches?"))
      return { total_count: fixture.versions.length, actions_caches: fixture.versions };
    if (route.startsWith("/pulls/")) return { state: fixture.states[route.split("/").pop()] };
    return {};
  }, true);
  assert.deepEqual(
    requests.filter(([method]) => method === "DELETE").map(([, route]) => route),
    ["/actions/caches/1", "/actions/caches/4"],
  );
  assert.ok(result.applied);
  const lastGet = requests.map(([method]) => method).lastIndexOf("GET");
  assert.ok(lastGet < requests.findIndex(([method]) => method === "DELETE"));
});

test("trusted events reject PR workflow runs, foreign repositories and non-main dispatches", () => {
  const repository = "jonathanong/no-mistakes";
  assert.ok(trustedEvent("workflow_run", fixture.events.workflow, repository, "refs/heads/main"));
  assert.ok(
    trustedEvent("pull_request_target", fixture.events.closed, repository, "refs/heads/main"),
  );
  assert.ok(trustedEvent("workflow_dispatch", {}, repository, "refs/heads/main"));
  assert.equal(trustedEvent("workflow_dispatch", {}, repository, "refs/heads/topic"), false);
  for (const field of ["event", "head_branch", "path"])
    assert.equal(
      trustedEvent(
        "workflow_run",
        { workflow_run: { ...fixture.events.workflow.workflow_run, [field]: "untrusted" } },
        repository,
        "refs/heads/main",
      ),
      false,
    );
  assert.equal(
    trustedEvent("workflow_run", fixture.events.workflow, "other/repository", "refs/heads/main"),
    false,
  );
  assert.equal(
    trustedEvent(
      "pull_request_target",
      { ...fixture.events.closed, action: "opened" },
      repository,
      "refs/heads/main",
    ),
    false,
  );
});

test("retention workflow executes only trusted sparse policy with isolated write permissions", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/cache-retention.yml"), "utf8");
  assert.match(workflow, /ref: \$\{\{ github.workflow_sha \}\}/);
  assert.match(workflow, /persist-credentials: false/);
  assert.match(workflow, /sparse-checkout: \.github\/scripts\/cache-retention.cjs/);
  assert.match(workflow, /actions: write/);
  assert.match(workflow, /types: \[closed\]/);
  assert.doesNotMatch(workflow, /pull_request\.head|npm install|schedule:|cancel-in-progress:/);
  assert.match(workflow, /runs-on: ubuntu-slim/);
  assert.match(workflow, /uses: pnpm\/setup@[a-f0-9]{40}/);
  assert.match(workflow, /version: 11\.24\.0/);
  assert.match(workflow, /runtime: node@24/);
  assert.match(workflow, /cache: false/);
  assert.match(workflow, /install: false/);
  assert.match(workflow, /workflow_call:[\s\S]*best_effort:[\s\S]*default: false/);
  assert.match(workflow, /continue-on-error: \$\{\{ inputs.best_effort \|\| false \}\}/);
});

test("malformed metadata and unresolved PR state never reach deletion", async () => {
  assert.throws(
    () => plan([...fixture.versions, fixture.versions[0]], fixture.states),
    /duplicate/,
  );
  const requests = [];
  await assert.rejects(
    retain(async (method, route) => {
      requests.push(method);
      if (route.startsWith("/actions/caches?"))
        return { total_count: fixture.versions.length, actions_caches: fixture.versions };
      return { state: "unknown" };
    }, true),
    /state is unknown/,
  );
  assert.equal(requests.includes("DELETE"), false);
});

test("reusable main-push callers must match both repository and payload branch", () => {
  const repository = "jonathanong/no-mistakes";
  assert.ok(trustedEvent("push", fixture.events.push, repository, "refs/heads/main"));
  assert.equal(
    trustedEvent("push", fixture.events.push, "foreign/repository", "refs/heads/main"),
    false,
  );
  assert.equal(trustedEvent("push", fixture.events.push, repository, "refs/heads/topic"), false);
  assert.equal(
    trustedEvent(
      "push",
      { ...fixture.events.push, ref: "refs/heads/topic" },
      repository,
      "refs/heads/main",
    ),
    false,
  );
  assert.equal(
    trustedEvent("push", { ref: "refs/heads/main" }, repository, "refs/heads/main"),
    false,
  );
});

test("API failures stay failures and publish unknown budget diagnostics for best-effort callers", async () => {
  await assert.rejects(
    retain(async () => {
      throw new Error("API unavailable");
    }, true),
    /API unavailable/,
  );
  assert.match(failureSummary("API unavailable"), /API unavailable/);
  assert.match(failureSummary("API unavailable"), /Budget unknown/);
});

test("benchmark handoff versions and verified Swatinem benchmark keys retain newest main records", () => {
  const result = plan(fixture.benchmarks, fixture.states, undefined, fixture.sourceRunStates);
  assert.deepEqual(
    result.remove.map((cache) => cache.id),
    [21, 31],
  );
  assert.deepEqual(
    result.kept.map((cache) => cache.id).sort((a, b) => a - b),
    [22, 32],
  );
  assert.equal(
    result.remove.some((cache) => cache.id === 23),
    false,
  );
  assert.deepEqual(
    plan(
      fixture.benchmarks.filter((cache) => !result.remove.some((item) => item.id === cache.id)),
      fixture.states,
      undefined,
      fixture.sourceRunStates,
    ).remove,
    [],
  );
});

test("only known retired main producer dependency copies are deleted", () => {
  const retired = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/cache-retention/retired-producer-dependencies.json"),
      "utf8",
    ),
  );
  const result = plan([...retired.obsolete, ...retired.protected], fixture.states);
  assert.deepEqual(
    result.remove.map((cache) => cache.id).sort((a, b) => a - b),
    retired.obsolete.map((cache) => cache.id).sort((a, b) => a - b),
  );
  for (const cache of retired.protected)
    assert.equal(
      result.remove.some((item) => item.id === cache.id),
      false,
    );
  assert.equal(result.remove.length, 6);
});

test("active and unknown handoff source runs are protected even beside newer completed caches", () => {
  for (const states of fixture.handoffLifecycle) {
    const result = plan(fixture.benchmarks, fixture.states, undefined, states);
    assert.equal(
      result.remove.some((cache) => [21, 22, 23].includes(cache.id)),
      false,
    );
  }
  const registry = plan(fixture.registry, {});
  assert.deepEqual(
    registry.remove.map((cache) => cache.id),
    [41],
  );
  assert.deepEqual(
    registry.kept.map((cache) => cache.id),
    [42],
  );
});

test("handoff source metadata is read once before any deletion and API failure fails closed", async () => {
  const calls = [];
  const request = async (method, route) => {
    calls.push([method, route]);
    if (route.startsWith("/actions/caches?"))
      return { total_count: fixture.benchmarks.length, actions_caches: fixture.benchmarks };
    if (route === "/pulls/3328") return { state: "open" };
    if (route.startsWith("/actions/runs/")) return fixture.sourceRuns[route.split("/").at(-1)];
    return {};
  };
  const result = await retain(request, true);
  assert.deepEqual(
    result.remove.map((cache) => cache.id),
    [21, 31],
  );
  assert.deepEqual(
    calls.filter(([method, route]) => route.startsWith("/actions/runs/")),
    [
      ["GET", "/actions/runs/100"],
      ["GET", "/actions/runs/101"],
    ],
  );
  const firstDelete = calls.findIndex(([method]) => method === "DELETE");
  assert.ok(firstDelete > calls.findIndex(([, route]) => route === "/actions/runs/101"));
  const failed = [];
  await assert.rejects(
    retain(async (method, route) => {
      failed.push(method);
      if (route.startsWith("/actions/runs/")) throw new Error("source metadata unavailable");
      return request(method, route);
    }, true),
    /source metadata unavailable/,
  );
  assert.equal(failed.includes("DELETE"), false);
});

test("deleted source runs preserve their handoffs without blocking unrelated cleanup", async () => {
  for (const failure of fixture.sourceRunFailures) {
    const deleted = [];
    const request = async (method, route) => {
      if (route.startsWith("/actions/caches?"))
        return { total_count: fixture.benchmarks.length, actions_caches: fixture.benchmarks };
      if (route === "/pulls/3328") return { state: "open" };
      if (route === "/actions/runs/100")
        throw Object.assign(new Error(failure.message), { status: failure.status });
      if (route === "/actions/runs/101") return fixture.sourceRuns[101];
      if (method === "DELETE") deleted.push(Number(route.split("/").at(-1)));
      return {};
    };
    if (failure.removals) {
      const result = await retain(request, true);
      assert.deepEqual(deleted, failure.removals);
      assert.equal(
        result.remove.some((cache) => cache.id === 21),
        false,
      );
    } else {
      await assert.rejects(retain(request, true), {
        message: failure.message,
        status: failure.status,
      });
      assert.deepEqual(deleted, []);
    }
  }
  const missing = fixture.sourceRunFailures[0];
  await assert.rejects(
    retain(async () => {
      throw Object.assign(new Error(missing.message), { status: missing.status });
    }, true),
    { status: 404 },
  );
});
