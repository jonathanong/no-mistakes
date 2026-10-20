import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { ids } from "./postgres-no-async-sql-builder-helpers.mjs";

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
