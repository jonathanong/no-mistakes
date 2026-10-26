import assert from "node:assert/strict";
import { it } from "vitest";
import { fixtureFile, require, tsParser } from "./async-source-session-shared.mjs";

const ts = require("typescript");
const { createSourceSession } = require("../src/rules/async-source-session");

function measuredSession(overrides = {}) {
  const reads = new Map();
  let configs = 0;
  let resolutions = 0;
  let parses = 0;
  const session = createSourceSession(
    {
      ...ts,
      sys: {
        ...ts.sys,
        readFile(path) {
          reads.set(path, (reads.get(path) || 0) + 1);
          return ts.sys.readFile(path);
        },
      },
      parseJsonConfigFileContent(...args) {
        configs++;
        return ts.parseJsonConfigFileContent(...args);
      },
      resolveModuleName(...args) {
        resolutions++;
        return ts.resolveModuleName(...args);
      },
      ...overrides,
    },
    {
      parseForESLint(...args) {
        parses++;
        return tsParser.parseForESLint(...args);
      },
    },
  );
  return { session, reads, counts: () => ({ configs, resolutions, parses }) };
}

it("shares configuration preparation and memoizes successful and failed resolutions", () => {
  const probe = measuredSession();
  const caller = fixtureFile("callers/caller0.ts");
  const consumer = fixtureFile("shared/consumers.ts");
  const job = fixtureFile("shared/jobs.ts");
  assert.equal(probe.session.resolve("@session/jobs", caller), job);
  assert.equal(probe.session.resolve("@session/jobs", caller), job);
  assert.equal(probe.session.resolve("@session/jobs", consumer), job);
  assert.equal(probe.session.resolve("@session/missing", caller), undefined);
  assert.equal(probe.session.resolve("@session/missing", caller), undefined);
  assert.deepEqual(probe.counts(), { configs: 1, resolutions: 3, parses: 0 });
  assert.equal(probe.reads.get(fixtureFile("tsconfig.json")), 1);
});

it("keeps distinct tsconfig resolver projections separate", () => {
  const probe = measuredSession();
  for (const project of ["projection-a", "projection-b"])
    assert.equal(
      probe.session.resolve("@session/jobs", fixtureFile(`${project}/caller.ts`)),
      fixtureFile(`${project}/jobs.ts`),
    );
  assert.deepEqual(probe.counts(), { configs: 2, resolutions: 2, parses: 0 });
});

it.each(["no-resolve", "invalid-config", "malformed-config"])(
  "memoizes a rejected %s configuration without attempting resolution",
  (name) => {
    const probe = measuredSession();
    const caller = fixtureFile(`${name}/caller.ts`);
    assert.equal(probe.session.resolve("@session/jobs", caller), null);
    assert.equal(probe.session.resolve("@session/jobs", caller), null);
    assert.equal(probe.counts().resolutions, 0);
    assert.equal(probe.reads.get(fixtureFile(`${name}/tsconfig.json`)), 1);
  },
);

it("uses Bundler defaults when no tsconfig is selected, including optional host methods", () => {
  const probe = measuredSession({
    findConfigFile: () => undefined,
    sys: { ...ts.sys, realpath: undefined },
  });
  assert.equal(
    probe.session.resolve("../shared/jobs", fixtureFile("callers/caller0.ts")),
    fixtureFile("shared/jobs.ts"),
  );
  assert.equal(probe.counts().configs, 0);
});

it("caches parsed sources and missing or invalid sources within the owner", () => {
  const probe = measuredSession();
  const job = fixtureFile("shared/jobs.ts");
  assert.equal(probe.session.load(job), probe.session.load(job));
  for (const name of ["bad-source.ts", "absent-source.ts"]) {
    const filename = fixtureFile(name);
    const first = probe.session.load(filename);
    assert.ok(probe.session.load(filename) === first);
    if (name === "bad-source.ts") assert.equal(first, null);
    assert.equal(probe.reads.get(filename), 1);
  }
  assert.equal(probe.counts().parses, 3);
  assert.equal(probe.reads.get(job), 1);
});
