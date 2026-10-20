import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { plugin } from "./helpers.mjs";
import { RULE, ids, reported } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("stays opt-in, inert without builders, and never autofixes", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.notEqual(plugin.rules[RULE].meta.hasSuggestions, true);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
    const code = "async function build() { return 'SELECT 1'; }\nread(build());";
    assert.deepEqual(ids(code, {}), []);
    assert.deepEqual(ids(code, { builders: [] }), []);
    assert.deepEqual(ids(code, { builders: [{ source: "@example/db" }] }), []);
    const hit = reported("async function build() { return 'SELECT 1'; }\nread(build());");
    assert.equal(hit.length, 1);
    assert.equal(hit[0].messageId, "removeAsync");
    assert.equal(hit[0].fix, undefined);
    assert.equal(hit[0].suggestions, undefined);
    assert.match(hit[0].message, /Remove async/);
    assert.match(hit[0].message, /No autofix/);
  });

  it("suggests removing async only for a local builder whose callers do not need a Promise", () => {
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function other() { return "SELECT 2"; }
        async function load() { return read(await build()); }
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const build = async () => "SELECT 1";
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return \`SELECT 1\`; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        type Sql = string;
        function accept(query: Sql) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function accept(query: string | number) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => (string & { length: number })) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string[]> { return ["SELECT 1"]; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<() => string> { return () => "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        void build();
        read(flag ? build() : "SELECT 2");
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build();
      `),
      ["removeAsync"],
    );
  });
});
