import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";
import { NAMES, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // reports only when every path returns judged SQL
    // A declared callee call is not a judged tag either.
    // Zero findings.
    {
      code: withSyncRead(`
          declare function sql(text: string): string;
          async function build() { return sql("SELECT 1"); }
          await build();
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) return;
          return "SELECT 1";
        }
        await build(false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        declare function loadSql(key: string): string;
        async function build() {
          return (0, loadSql("k"));
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          const left = right;
          const right = left;
          return left;
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          let query = "SELECT 1";
          query = "SELECT 2";
          return query;
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // recalls a module-level string or constant template and ignores other module expressions
    {
      code: withSyncRead(`
        const Q = sql\`SELECT 1\`;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = flag ? "SELECT 1" : "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT " + "1";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = \`SELECT \${1}\`;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        let Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = 1;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        Q = "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function outer() {
          const Q = "SELECT 1";
          async function build() { return Q; }
          read(build());
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const { Q } = { Q: "SELECT 1" };
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        declare const Q: string;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // build() runs before Q's initializer, still in the same statement.
    // Zero findings.
    {
      code: withSyncRead(`
        const result = build(), Q = "SELECT 1";
        async function build() { return Q; }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Calling the hoisted builder before the const throws once async is gone.
    // Zero findings.
    {
      code: withSyncRead(`
        async function build() { return Q; }
        build();
        after();
        const Q = "SELECT 1";
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const build = async () => Q;
        build();
        const Q = "SELECT 1";
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/callback-before.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/hoisted-early.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-init.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/hoisted-class.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/class-call-before.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-class.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-class-const.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-class-specifier.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-default-class.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Anonymous default function declarations are hoisted. A cycle can call
    // them, and a callback inside them, before the const.
    // Zero findings.
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-default-fn.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-default-async-fn.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/exported-default-fn-callback.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/cycle.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // A typed callback still decides. An unannotated callee stays unknown.
    // Zero findings.
    {
      code: withSyncRead(`
        function run(load: () => Promise<string>) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        [1].map((): Promise<string> => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // A bound arrow is not a callback argument, so its promise stays unknown.
    // Zero findings.
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const load = () => build();
        read(load());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // A returned array keeps its element promises. A discarded map still reports.
    // Zero findings.
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/returned-array-map.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Only an array literal's discarded map is a known callback contract.
    // Zero findings.
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const name = "map";
        consumer.map(() => build());
        [1][name](() => build());
        [1]?.map(() => build());
        [1].forEach(() => build());
        (0, consumer)(() => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function take(rows) { return rows; }
        const Q = "SELECT 1";
        async function build() { return Q; }
        take([1].map(() => build()));
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // The arrow exists before Q and is exported, so a cycle can call it first.
    // Zero findings.
    {
      code: withSyncRead(`
        const init = () => { build(); };
        trigger();
        const Q = "SELECT 1";
        async function build() { return Q; }
        export { init };
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const init = () => { build(); };
        const Q = "SELECT 1";
        async function build() { return Q; }
        init();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // An exported helper is hoisted, so a cyclic import can call it before the const.
    // Zero findings.
    {
      code: withSyncRead(
        'export function invoke() { build(); }\nconst Q = "SELECT 1";\nasync function build() { return Q; }\nread(invoke());',
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        'function invoke() { build(); }\nexport { invoke };\nconst Q = "SELECT 1";\nasync function build() { return Q; }',
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // invoke() runs before the const even though its build reference is below it.
    // Zero findings.
    {
      code: withSyncRead(`
        invoke();
        after();
        const Q = "SELECT 1";
        function invoke() { build(); }
        async function build() { return Q; }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: string) { return id; }
        read(build("SELECT 1"));
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return missingSql; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const A = B;
        const B = A;
        async function build() { return A; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        const Q = "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const INNER = call();
        const Q = INNER;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // reports only when every path returns judged SQL
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          else return "SELECT 2";
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          }
          return "SELECT 2";
        }
        await build(false, false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) {
            const label = "kept";
            void label;
          }
          return "SELECT 1";
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) {}
          else {}
          return "SELECT 1";
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) {}
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          } else return "SELECT 2";
          return "SELECT 3";
        }
        await build(false, false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          done: return "SELECT 1";
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          function nested() {}
          class Box {}
          ;
          debugger;
          return \`SELECT 1\`;
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          const query = "SELECT 1";
          return query;
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          return (0, "SELECT 1");
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          return "SELECT 1" && "SELECT 2";
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const build = async () => "SELECT 1";
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // recalls a module-level string or constant template and ignores other module expressions
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = \`SELECT 1\`;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const INNER = "SELECT 1";
        const Q = INNER;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1" as string;
        async function build() { return Q; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build(flag: boolean) { return flag ? Q : "SELECT 2"; }
        read(build(true));
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // The call is after the initializer, so the hoisted function is still judged.
    // One removeAsync finding.
    {
      code: withSyncRead(`
        async function build() { return Q; }
        const Q = "SELECT 1";
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // A later declarator in the same statement runs after Q's initializer.
    // One removeAsync finding.
    {
      code: withSyncRead(`
        const Q = "SELECT 1", result = build();
        async function build() { return Q; }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const INNER = "SELECT 1", Q = INNER, result = build();
        async function build() { return Q; }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // later() is only called after the const, so the nested call is judged.
    // One removeAsync finding.
    {
      code: withSyncRead(`
        function later() { build(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        read(later());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Saved fixtures. The issue example returns the call from map and reports.
    // A callback before the const stays silent. A callback inside a hoisted
    // or exported function is judged by that function, not by its own position.
    // One removeAsync finding.
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/callback-after.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/hoisted-after.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/class-call-after.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/class-field-after.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/anonymous-class.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: () => string) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        [1].map(async () => build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(fixture(`postgres-no-async-sql-builder/discarded-array-map-async.ts`)),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const init = () => { build(); };
        export { init };
        init();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const init = () => { build(); };
        const Q = "SELECT 1";
        async function build() { return Q; }
        init();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Q = "SELECT 1";
        function invoke() { build(); }
        async function build() { return Q; }
        read(invoke());
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-returns.snap");
}, 30_000);
