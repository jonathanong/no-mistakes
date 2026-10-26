import assert from "node:assert/strict";
import { it } from "vitest";
import { cli, fixtureFile, require, withArgv } from "./async-source-session-shared.mjs";

const {
  oneShotCli,
  sourceOwner,
  sourcesUnavailable,
  withSourceOwner,
} = require("../src/rules/async-source-owner");

it("recognizes only the canonical public npm CLI entry", () => {
  assert.equal(oneShotCli(cli, []), true);
  assert.equal(oneShotCli("", []), false);
  for (const name of ["unknown-host", "invalid-host", "nonstring-bin", "mismatched-bin"])
    assert.equal(oneShotCli(fixtureFile(`${name}/bin/host.cjs`), []), false, name);
  assert.equal(oneShotCli(fixtureFile("nonexistent-host.cjs"), []), false);
});

it.each([
  "--lsp",
  "--lsp=true",
  "--watch",
  "--watch=true",
  "--fix",
  "--fix-suggestions",
  "--fix-dangerously",
])("keeps persistent or mutating CLI mode %s isolated", (flag) =>
  assert.equal(oneShotCli(cli, [flag]), false),
);

it("shares owned facts while recreating and releasing every file visitor", () => {
  withArgv(cli, [], () => {
    const context = { filename: "first.ts" };
    const owners = [];
    const unavailable = [];
    const visited = [];
    const hooks = withSourceOwner({
      create(ownerContext) {
        owners.push(sourceOwner(ownerContext));
        unavailable.push(sourcesUnavailable(ownerContext));
        const filename = ownerContext.filename;
        return {
          Program() {
            visited.push(filename);
          },
        };
      },
    }).createOnce(context);
    assert.equal(hooks.before(), undefined);
    hooks.Program();
    hooks.CallExpression(); // Absent ordinary handlers are safe to dispatch.
    owners[0].sources = false;
    hooks.after();
    assert.equal(sourceOwner(context), null);
    assert.equal(sourcesUnavailable(context), false);
    context.filename = "second.ts";
    hooks.before();
    hooks.Program();
    hooks.after();
    assert.equal(owners[0], owners[1]);
    assert.deepEqual(unavailable, [false, true]);
    assert.deepEqual(visited, ["first.ts", "second.ts"]);
    assert.equal(sourceOwner(context), null);
  });
});

it.each(["--lsp", "--fix", "--watch"])("does not share an owner in %s mode", (flag) => {
  withArgv(cli, [flag], () => {
    const context = {};
    const hooks = withSourceOwner({
      create() {
        assert.equal(sourceOwner(context), null);
        return { Program() {} };
      },
    }).createOnce(context);
    for (let file = 0; file < 2; file++) {
      hooks.before();
      hooks.after();
    }
  });
});

it("skips the host AST walk and releases the association for an inactive rule", () => {
  const context = {};
  const hooks = withSourceOwner({ create: () => ({}) }).createOnce(context);
  assert.equal(hooks.before(), false);
  // Oxlint does not run after when before returns false.
  assert.equal(sourceOwner(context), null);
});

it("releases an association when creating the file visitor throws", () => {
  const context = {};
  const hooks = withSourceOwner({
    create() {
      throw new Error("Invalid file configuration");
    },
  }).createOnce(context);
  assert.throws(() => hooks.before(), /Invalid file configuration/);
  assert.equal(sourceOwner(context), null);
});
