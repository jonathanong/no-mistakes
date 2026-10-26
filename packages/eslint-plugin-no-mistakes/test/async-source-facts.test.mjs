import assert from "node:assert/strict";
import { it, vi } from "vitest";
import { messages } from "./helpers.mjs";
import {
  fixtureFile,
  fixtureText,
  instrumentSources,
  require,
  targets,
  tsParser,
} from "./async-source-session-shared.mjs";

const { parsedSource, currentSources } = require("../src/rules/async-source-files");
const { createSourceSession } = require("../src/rules/async-source-session");
const { createSourceTypes } = require("../src/rules/async-source-types");
const { compileTargets } = require("../src/rules/async-patterns");
const { createDeclarations } = require("../src/rules/async-source-declarations");
const ts = require("typescript");
const compiled = compileTargets({ targets }, "targets");

function prepared() {
  const filename = fixtureFile("typed-targets.ts");
  const source = parsedSource(filename, fixtureText("typed-targets.ts"), tsParser);
  return {
    context: source.context,
    sources: currentSources(source.context, createSourceSession(ts, tsParser)),
    ast: source.ast,
  };
}

function find(source, type, name) {
  const pending = [source.ast];
  while (pending.length) {
    const node = pending.pop();
    if (node.type === type && (!name || node.name === name)) return node;
    for (const key of source.context.sourceCode.visitorKeys[node.type] || []) {
      const value = node[key];
      pending.push(...(Array.isArray(value) ? value.filter(Boolean) : value ? [value] : []));
    }
  }
  throw new Error(`Fixture has no ${type} ${name || ""}`);
}

it("skips source work for array helpers, primitive/function annotations, and unsupported factory shapes", () => {
  const probe = instrumentSources();
  try {
    assert.deepEqual(
      messages(
        fixtureText("unrelated.ts"),
        "async-call-disposition",
        { targets },
        fixtureFile("unrelated.ts"),
      ),
      [],
    );
    assert.equal(probe.stats.reads.size, 0);
    assert.equal(probe.stats.parses, 0);
    assert.equal(probe.stats.resolutions, 0);
  } finally {
    probe.restore();
  }
});

it("retains inferred factory aliases and typed target calls without an inline producer", () => {
  assert.deepEqual(
    messages(
      fixtureText("typed-targets.ts"),
      "async-call-disposition",
      { targets },
      fixtureFile("typed-targets.ts"),
    ),
    ["disposition", "disposition", "disposition"],
  );
});

it("indexes imported exports once while keeping configured target decisions separate", () => {
  const source = prepared();
  const imported = source.ast.body[0].specifiers[0].local;
  const owned = source.sources.load(fixtureFile("shared/jobs.ts"));
  let scans = 0;
  const iterator = owned.ast.body[Symbol.iterator];
  const scan = vi.spyOn(owned.ast.body, Symbol.iterator).mockImplementation(function () {
    scans++;
    return iterator.call(this);
  });
  try {
    const first = createSourceTypes(compiled, source.sources).target(imported, source.context);
    const second = createSourceTypes(compiled, source.sources).target(imported, source.context);
    assert.ok(first && first === second);
    assert.equal(scans, 1);
    assert.equal(createSourceTypes([], source.sources).target(imported, source.context), null);
  } finally {
    scan.mockRestore();
  }
});

it("memoizes complete type queries without reusing them across source projections", () => {
  const source = prepared();
  let lookups = 0;
  const sources = {
    ...source.sources,
    resolve(...args) {
      lookups++;
      return source.sources.resolve(...args);
    },
  };
  const proof = createSourceTypes(compiled, sources);
  const imported = source.ast.body[0].specifiers[0].local;
  const type = find(source, "TSTypeQuery");
  const member = find(source, "MemberExpression");
  for (const [method, node] of [
    ["target", imported],
    ["targetType", type],
    ["valueType", member],
  ]) {
    const first = proof[method](node, source.context);
    assert.ok(first);
    const before = lookups;
    assert.ok(proof[method](node, source.context) === first);
    assert.equal(lookups, before);
  }
  assert.equal(proof.target(null, source.context), null);
  assert.equal(proof.targetType(null, source.context), null);
  assert.equal(proof.valueType(null, source.context), null);
  assert.equal(createSourceTypes([], sources).targetType(type, source.context), null);
});

it.each([
  ["wide-caller.ts", 128],
  ["wide-unreadable-caller.ts", 2],
])("bounds star-export resolution for %s", (name, expectedResolutions) => {
  const filename = fixtureFile(name);
  const source = parsedSource(filename, fixtureText(name), tsParser);
  const store = createSourceSession(ts, tsParser);
  let resolutions = 0;
  const sources = currentSources(source.context, {
    ...store,
    resolve(...args) {
      resolutions++;
      return store.resolve(...args);
    },
  });
  const result = createDeclarations(sources).declaration(
    source.ast.body[0].specifiers[0].local,
    source.context,
  );
  assert.equal(result, false);
  // Include the initial caller-to-barrel edge in the observed work count.
  assert.equal(resolutions, expectedResolutions);
});

it("links scopes when a compatible parser omits visitor keys for a leaf", () => {
  const source = parsedSource(fixtureFile("shared/jobs.ts"), fixtureText("shared/jobs.ts"), {
    parseForESLint(...args) {
      const parsed = tsParser.parseForESLint(...args);
      return { ...parsed, visitorKeys: { ...parsed.visitorKeys, Literal: undefined } };
    },
  });
  const literal = find(source, "Literal");
  assert.ok(
    source.context.sourceCode.getScope(literal).variables.some(({ name }) => name === "holes"),
  );
});
