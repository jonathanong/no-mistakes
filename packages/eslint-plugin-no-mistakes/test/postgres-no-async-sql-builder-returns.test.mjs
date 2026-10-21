import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture } from "./helpers.mjs";
import { ids } from "./postgres-no-async-sql-builder-helpers.mjs";

function saved(name) {
  return ids(fixture(`postgres-no-async-sql-builder/${name}`));
}

describe("postgres-no-async-sql-builder", () => {
  it("reports only when every path returns judged SQL", () => {
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          else return "SELECT 2";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          }
          return "SELECT 2";
        }
        await build(false, false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {
            const label = "kept";
            void label;
          }
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {}
          else {}
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {}
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          } else return "SELECT 2";
          return "SELECT 3";
        }
        await build(false, false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          done: return "SELECT 1";
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          function nested() {}
          class Box {}
          ;
          debugger;
          return \`SELECT 1\`;
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          const query = "SELECT 1";
          return query;
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          return (0, "SELECT 1");
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          return "SELECT 1" && "SELECT 2";
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const build = async () => "SELECT 1";
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          declare function sql(text: string): string;
          async function build() { return sql("SELECT 1"); }
          await build();
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      // A declared callee call is not a judged tag either.
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return;
          return "SELECT 1";
        }
        await build(false);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        declare function loadSql(key: string): string;
        async function build() {
          return (0, loadSql("k"));
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          const left = right;
          const right = left;
          return left;
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          let query = "SELECT 1";
          query = "SELECT 2";
          return query;
        }
        await build();
      `),
      [],
    );
  });

  it("recalls a module-level string or constant template and ignores other module expressions", () => {
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const Q = \`SELECT 1\`;
        async function build() { return Q; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const INNER = "SELECT 1";
        const Q = INNER;
        async function build() { return Q; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        export const Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1" as string;
        async function build() { return Q; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build(flag: boolean) { return flag ? Q : "SELECT 2"; }
        read(build(true));
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const Q = sql\`SELECT 1\`;
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = flag ? "SELECT 1" : "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT " + "1";
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = \`SELECT \${1}\`;
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        let Q = "SELECT 1";
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = 1;
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        Q = "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function outer() {
          const Q = "SELECT 1";
          async function build() { return Q; }
          read(build());
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const { Q } = { Q: "SELECT 1" };
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        declare const Q: string;
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    // The call is after the initializer, so the hoisted function is still judged.
    assert.deepEqual(
      ids(`
        async function build() { return Q; }
        const Q = "SELECT 1";
        read(build());
      `),
      ["removeAsync"],
    );
    // A later declarator in the same statement runs after Q's initializer.
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1", result = build();
        async function build() { return Q; }
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const INNER = "SELECT 1", Q = INNER, result = build();
        async function build() { return Q; }
      `),
      ["removeAsync"],
    );
    // build() runs before Q's initializer, still in the same statement.
    assert.deepEqual(
      ids(`
        const result = build(), Q = "SELECT 1";
        async function build() { return Q; }
      `),
      [],
    );
    // Calling the hoisted builder before the const throws once async is gone.
    assert.deepEqual(
      ids(`
        async function build() { return Q; }
        build();
        after();
        const Q = "SELECT 1";
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const build = async () => Q;
        build();
        const Q = "SELECT 1";
      `),
      [],
    );
    // later() is only called after the const, so the nested call is judged.
    assert.deepEqual(
      ids(`
        function later() { build(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        read(later());
      `),
      ["removeAsync"],
    );
    // Saved fixtures. The issue example returns the call from map and reports.
    // A callback before the const stays silent. A callback inside a hoisted
    // or exported function is judged by that function, not by its own position.
    assert.deepEqual(saved("callback-after.ts"), ["removeAsync"]);
    assert.deepEqual(saved("callback-before.ts"), []);
    assert.deepEqual(saved("hoisted-early.ts"), []);
    assert.deepEqual(saved("hoisted-after.ts"), ["removeAsync"]);
    assert.deepEqual(saved("exported-init.ts"), []);
    assert.deepEqual(saved("hoisted-class.ts"), []);
    assert.deepEqual(saved("class-call-before.ts"), []);
    assert.deepEqual(saved("class-call-after.ts"), ["removeAsync"]);
    assert.deepEqual(saved("class-field-after.ts"), ["removeAsync"]);
    assert.deepEqual(saved("exported-class.ts"), []);
    assert.deepEqual(saved("exported-class-const.ts"), []);
    assert.deepEqual(saved("exported-class-specifier.ts"), []);
    assert.deepEqual(saved("exported-default-class.ts"), []);
    assert.deepEqual(saved("anonymous-class.ts"), ["removeAsync"]);
    // Anonymous default function declarations are hoisted. A cycle can call
    // them, and a callback inside them, before the const.
    assert.deepEqual(saved("exported-default-fn.ts"), []);
    assert.deepEqual(saved("exported-default-async-fn.ts"), []);
    assert.deepEqual(saved("exported-default-fn-callback.ts"), []);
    assert.deepEqual(saved("cycle.ts"), []);
    // A typed callback still decides. An unannotated callee stays unknown.
    assert.deepEqual(
      ids(`
        function run(load: () => Promise<string>) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => string) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load) { return load(); }
        const Q = "SELECT 1";
        async function build() { return Q; }
        run(() => build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        [1].map((): Promise<string> => build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        [1].map(async () => build());
      `),
      ["removeAsync"],
    );
    // A bound arrow is not a callback argument, so its promise stays unknown.
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const load = () => build();
        read(load());
      `),
      [],
    );
    // Only an array literal's discarded map is a known callback contract.
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const name = "map";
        consumer.map(() => build());
        [1][name](() => build());
        [1]?.map(() => build());
        [1].forEach(() => build());
        (0, consumer)(() => build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function take(rows) { return rows; }
        const Q = "SELECT 1";
        async function build() { return Q; }
        take([1].map(() => build()));
      `),
      [],
    );
    // The arrow exists before Q and is exported, so a cycle can call it first.
    assert.deepEqual(
      ids(`
        const init = () => { build(); };
        trigger();
        const Q = "SELECT 1";
        async function build() { return Q; }
        export { init };
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const init = () => { build(); };
        const Q = "SELECT 1";
        async function build() { return Q; }
        init();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        async function build() { return Q; }
        const init = () => { build(); };
        export { init };
        init();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const init = () => { build(); };
        const Q = "SELECT 1";
        async function build() { return Q; }
        init();
      `),
      ["removeAsync"],
    );
    // An exported helper is hoisted, so a cyclic import can call it before the const.
    assert.deepEqual(
      ids(
        'export function invoke() { build(); }\nconst Q = "SELECT 1";\nasync function build() { return Q; }\nread(invoke());',
      ),
      [],
    );
    assert.deepEqual(
      ids(
        'function invoke() { build(); }\nexport { invoke };\nconst Q = "SELECT 1";\nasync function build() { return Q; }',
      ),
      [],
    );
    // invoke() runs before the const even though its build reference is below it.
    assert.deepEqual(
      ids(`
        invoke();
        after();
        const Q = "SELECT 1";
        function invoke() { build(); }
        async function build() { return Q; }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        function invoke() { build(); }
        async function build() { return Q; }
        read(invoke());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(id: string) { return id; }
        read(build("SELECT 1"));
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return missingSql; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const A = B;
        const B = A;
        async function build() { return A; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Q = "SELECT 1";
        const Q = "SELECT 2";
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const INNER = call();
        const Q = INNER;
        async function build() { return Q; }
        read(build());
      `),
      [],
    );
  });
});
