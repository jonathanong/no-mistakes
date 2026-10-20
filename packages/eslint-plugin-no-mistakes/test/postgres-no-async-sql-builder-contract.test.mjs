import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";
import { NAMES, RULE, SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("abstains when the visible contract requires a Promise", () => {
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> | null { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): (Promise<string>) { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Sql = Promise<string>;
        async function build(): Sql { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): globalThis.Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => Promise<string>) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Loader = () => Promise<string>;
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => string) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => string = () => "SELECT 2") { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: { new (text: string): string }) { return new load("SELECT 1"); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Loader = { (): string };
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build().then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build()?.then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const query: Promise<string> = build();
        read(query);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function load(): Promise<string> { return build(); }
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          function run(load: () => Promise<string>) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          function run(load: () => string) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `,
        SQL,
      ),
      ["removeAsync"],
    );
  });

  it("abstains when the contract cannot be proved", () => {
    assert.deepEqual(
      ids(`
        export async function build() { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        export { build };
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const build = async () => "SELECT 1";
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Box = { sql: string };
        async function build(): Box["sql"] { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Sql = string;
        function wrap() {
          type Sql = Promise<string>;
          async function build(): Sql { return "SELECT 1"; }
          read(build());
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        import { run } from "./runner";
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function load() { return build(); }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build()?.toString();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        let build;
        build = async function () { return "SELECT 1"; };
        read(build());
      `),
      [],
    );
  });

  it("abstains for any, unknown, and object, and still reports string", () => {
    assert.deepEqual(
      messages(
        `
          function wrap(p: any) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function wrap(p: unknown) { return (p as Promise<string>).then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function wrap(p: object) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function run2(cb: any) { return cb().then(String); }
          async function build() { return "SELECT 1"; }
          run2(build);
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function run2(cb: () => any) { return cb().then(String); }
          async function build() { return "SELECT 1"; }
          run2(build);
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(query: string) { return query; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(): any { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const query: any = build();
        async function build() { return "SELECT 1"; }
        read(query);
      `),
      [],
    );
  });

  it("abstains when the result is only tested for truthiness", () => {
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build() ? read("a") : read("b");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build() || read("a");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const s = await build();
        read(s);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        await (flag || build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        read((0, build()));
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        read(build() as string);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        await build?.();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function outer() { return async function build() { return "SELECT 1"; }; }
        read(outer()());
      `),
      [],
    );
  });
});
