import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { NAMES, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // abstains on promise-compatible type literals and still reports a required member
    {
      code: withSyncRead(`
        function run2(cb: () => { then(f: any): any }) { return cb().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => {}) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type T = {};
        function run2(cb: () => T) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { (): {} }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { foo?: string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { [key: string]: string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { toString(): string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { valueOf(): object }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { constructor: Function }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { hasOwnProperty(k: PropertyKey): boolean }) {
          return (cb() as any).then(String);
        }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { call(t: any): any }) { return cb.call(null).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { apply(t: any, a?: any): any }) { return cb.apply(null).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { bind(t: any): () => any }) { return cb.bind(null)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { length: number }) { return (cb as any)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          class A { load: { name: string } = async () => "SELECT 1"; }
        `),
      filename: "app.ts",
      options: [{ builders: [{ names: ["load"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { then: any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { catch(f: any): any }) { return p.catch(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { finally(f: any): any }) { return p.finally(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        const p = build();
        wrap(p);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type T = { then(f: any): any };
        function wrap(p: T) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { then(f: any): any } | string) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        async function outer() { return wrap(build()); }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { then: any }) { return (cb as any)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface L { name: string }
        function run(l: L) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Child extends Missing { name: string }
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface A extends B {}
        interface B extends A {}
        function run(l: A) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Child extends Foo.Bar { name: string }
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): string | {} { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: {}) { return (p as any).then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          const box: { f: () => {} } = { async f() { return "SELECT 1"; } };
          read(box.f());
        `),
      filename: "app.ts",
      options: [{ builders: [{ names: ["f"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          const box: { f: () => { foo: string } } = { async f() { return "SELECT 1"; } };
          read(box.f());
        `),
      filename: "app.ts",
      options: [{ builders: [{ names: ["f"] }] }],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // abstains on promise-compatible type literals and still reports a required member
    {
      code: withSyncRead(`
        function run2(cb: () => { foo: string }) { return cb(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: { foo: string }) { return (cb as any)(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run2(cb: () => { toString(): string; foo: string }) { return cb(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { foo: string }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { name: string }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { length: number }) { return p.length; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface L { foo: string }
        function run(l: L) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Parent { foo: string }
        interface Child extends Parent {}
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(p: { f: () => {} }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-literals.snap");
}, 30_000);
