import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "vitest";
import { memberKey, mutationCall, staticKey } from "../src/rules/no-object-mutation-syntax.js";
import { __dirname, lint, messages, plugin } from "./helpers.mjs";

const RULE = "no-object-mutation";

function ids(code, option, filename = "fixture.ts") {
  return messages(code, RULE, option, filename);
}

function repeated(count) {
  return Array.from({ length: count }, () => "mutation");
}

describe("no-object-mutation", () => {
  it("stays out of the recommended and strict presets", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });

  it("treats a non-member as an absent key", () => {
    assert.equal(memberKey({ type: "Identifier" }), null);
    assert.equal(
      memberKey({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: "id" },
      }),
      "id",
    );
    assert.equal(
      memberKey({
        type: "MemberExpression",
        computed: false,
        property: { type: "Literal", value: "id" },
      }),
      null,
    );
    assert.equal(staticKey({ type: "Literal", value: "id" }), "id");
    assert.equal(staticKey({ type: "Literal", value: true }), null);
    const scope = { cells: new Map(), outer: null };
    const objectAssign = {
      type: "MemberExpression",
      computed: false,
      object: { type: "Identifier", name: "Object" },
      property: { type: "Identifier", name: "assign" },
    };
    assert.equal(
      mutationCall(
        { callee: objectAssign, arguments: [{ type: "Identifier", name: "rows" }] },
        scope,
      )?.targetNode.type,
      "Identifier",
    );
    assert.equal(mutationCall({ callee: objectAssign, arguments: [] }, scope), null);
    assert.equal(
      mutationCall(
        {
          callee: objectAssign,
          arguments: [{ type: "SpreadElement", argument: { type: "Identifier", name: "rows" } }],
        },
        scope,
      ),
      null,
    );
  });

  it("names the static rewrite in the report", () => {
    const [message] = lint("const tables = {};\ntables.a = 1;\n", {
      "no-mistakes/no-object-mutation": "error",
    });
    assert.match(message.message, /statically/);
    assert.doesNotMatch(message.message, /spread/);
    assert.match(message.message, /Declare a literal once/);
  });

  it("reports module member writes, updates, and computed keys", () => {
    assert.deepEqual(
      ids(`
        const tables = {};
        tables.a = 1;
        tables["b"] = 2;
        tables[\`c\`] = 3;
        tables[1] = 4;
        tables[key] = 5;
        tables.a += 1;
        tables.a++;
        ++tables.b;
        let n = 0;
        n++;
        const { a } = tables;
        delete tables.a;
      `),
      repeated(8),
    );
  });

  it("reports imports, namespaces, and reachable members", () => {
    assert.deepEqual(
      ids(`
        import def, { named, type TypeOnly } from "./mod";
        import * as ns from "./mod";
        def.a = 1;
        named.b = 1;
        ns.c = 1;
        const client = named.client;
        client.get = 1;
        export { named as alias };
        export * from "./mod";
      `),
      repeated(4),
    );
  });

  it("keeps an import shared when it is reassigned", () => {
    assert.deepEqual(
      ids(`
        import value from "./mod";
        value = {};
        value.a = 1;
      `),
      ["mutation"],
    );
  });

  it("reports parameters, aliases, elements, and parameter properties", () => {
    assert.deepEqual(
      ids(`
        function write(box, rows, ...parts) {
          box.a = 1;
          const client = box.client;
          client.get = 1;
          const { nested } = box;
          nested.id = 1;
          const [row] = rows;
          row.a = 1;
          parts.push(1);
          for (const item of rows) item.a = 1;
          for (var legacy of rows) legacy.a = 1;
        }
        class Service {
          constructor(public box) {
            box.ready = true;
          }
        }
      `),
      repeated(8),
    );
  });

  it("allows a parameter rebound to a fresh object", () => {
    assert.deepEqual(
      ids(`
        function build(box = {}) {
          box.a = 1;
          box = {};
          box.b = 2;
          return box;
        }
      `),
      ["mutation"],
    );
  });

  it("allows a local builder and reports writes after escape", () => {
    assert.deepEqual(
      ids(`
        function build(extra) {
          const box = {};
          box.client = { get: extra };
          box.client.get = extra;
          const rows = [];
          rows.push(1);
          return box;
        }
        function leaked(sink, cond, tag) {
          const box = {};
          sink(box);
          box.a = 1;
          const other = {};
          if (cond) sink(other);
          other.a = 1;
          const thrown = {};
          throw thrown;
          thrown.a = 1;
          const yielded = {};
          function* gen() { yield yielded; }
          const tagged = {};
          tag\`\${tagged}\`;
          tagged.a = 1;
          const stored = {};
          sink.slot = stored;
          stored.a = 1;
        }
      `),
      repeated(6),
    );
  });

  it("reports a fresh object stored into an outer or module binding", () => {
    assert.deepEqual(
      ids(`
        let g;
        function intoModule() {
          const b = {};
          g = b;
          b.a = 1;
        }
        function intoArray() {
          const b = {};
          g = [b];
          b.a = 1;
        }
        function intoOuter() {
          let h;
          function inner() {
            const b = {};
            h = b;
            b.a = 1;
          }
        }
        function sameFunction() {
          let h;
          const b = {};
          h = b;
          b.a = 1;
          return h;
        }
      `),
      repeated(3),
    );
  });

  it("reports a closure that escapes inside a value", () => {
    assert.deepEqual(
      ids(`
        function method() {
          const b = {};
          return { set(v) { b.a = v; } };
        }
        function pair() {
          const b = {};
          const g = () => { b.a = 1; };
          return [b, g];
        }
        function shorthand() {
          const b = {};
          const g = () => { b.a = 1; };
          return { g };
        }
        function sunk(sink) {
          const b = {};
          const g = () => { b.a = 1; };
          sink([g]);
        }
        let h;
        function stored() {
          const b = {};
          const g = () => { b.a = 1; };
          h = g;
        }
        function localCall() {
          const b = {};
          const g = () => { b.a = 1; };
          g();
          return b;
        }
        function localMethod() {
          const b = {};
          const obj = { set(v) { b.a = v; } };
          obj;
          return b;
        }
      `),
      repeated(5),
    );
  });

  it("reports a write into a value carried across loop iterations", () => {
    assert.deepEqual(
      ids(`
        const x = {};
        const cache = new Map();
        function carried() {
          let unused;
          let y = {};
          for (let i = 0; i < 2; i++) {
            y.a = 1;
            y = x;
          }
          let z = {};
          while (x) {
            z.a = 1;
            z = x;
          }
          let a = {};
          let b = {};
          function nested() {}
          for (let i = 0; i < 3; i++) {
            a.p = 1;
            a = b;
            b = x;
          }
        }
        function localOnly() {
          let y = {};
          for (let i = 0; i < 2; i++) {
            y.a = 1;
            y = {};
            y = cache;
            y = this;
          }
        }
      `),
      repeated(3),
    );
    // The escaped object is created inside the loop, so carried ranks stay
    // put and only the escape counter asks for another pass.
    assert.deepEqual(
      ids(`
        function innerEscape() {
          for (let i = 0; i < 2; i++) {
            const box = {};
            return box;
          }
        }
      `),
      [],
    );
  });

  it("keeps a called local closure from escaping its builder", () => {
    // Calling the closure runs its writes. It does not escape the builder
    // unless the closure itself escapes or the body passes the object out.
    assert.deepEqual(
      ids(`
        function build() {
          function set() { box.a = 1; }
          const box = {};
          set();
          box.b = 2;
          return box;
        }
      `),
      [],
    );
  });

  it("reports a dynamic read that can reach a shared field", () => {
    assert.deepEqual(
      ids(`
        function edit(param, key, other) {
          const box = {};
          box.user = param;
          box[other] = 1;
          box[key].name = "a";
          const local = {};
          local[key] = 1;
        }
      `),
      ["mutation"],
    );
  });

  it("lints an Espree class that has no decorators list", () => {
    assert.deepEqual(
      ids(
        `
        class Box {
          set(value) { this.slot = value; }
        }
      `,
        undefined,
        "fixture.js",
      ),
      [],
    );
  });

  it("reports a closure that escapes or runs after the object escapes", () => {
    assert.deepEqual(
      ids(`
        function publish(sink) {
          const box = {};
          return function set() { box.a = 1; };
        }
        function after(sink) {
          const box = {};
          function set() { box.a = 1; }
          sink(box);
          set();
        }
        function passed(sink) {
          const box = {};
          sink(function () { box.a = 1; });
          sink(function () { box.b = 2; });
        }
      `),
      repeated(4),
    );
  });

  it("reports the next loop iteration after an escape", () => {
    assert.deepEqual(
      ids(`
        function walk(xs, sink) {
          const box = {};
          for (const item of xs) {
            box.a = 1;
            sink(box);
          }
          for (let i = 0; i < xs.length; i++) {
            box.b = 1;
            sink(box);
          }
          while (xs.length) {
            box.c = 1;
            sink(box);
          }
          do {
            box.d = 1;
            sink(box);
          } while (xs.length);
        }
      `),
      repeated(4),
    );
  });

  it("allows own fields of a fresh copy and reports a shared nested field", () => {
    assert.deepEqual(
      ids(`
        function copyOf(box) {
          const copy = { ...box, extra: 1 };
          copy.extra = 2;
          copy.client.get = 3;
          const local = { client: {} };
          local.client.get = 4;
          return local;
        }
      `),
      ["mutation"],
    );
  });

  it("reports a shared value joined with another shared or fresh value", () => {
    assert.deepEqual(
      ids(`
        import { left, right } from "./mod";
        function pick(flag, param) {
          const target = flag ? left : right;
          target.a = 1;
          const same = flag ? param : param;
          same.a = 1;
          const mixed = flag ? param : {};
          mixed.a = 1;
          const fromFresh = flag ? {} : param;
          fromFresh.a = 1;
          const either = left && right;
          either.a = 1;
          const chain = flag ? left : right;
          (chain).a = 1;
          const bound = identity<string>;
        }
      `),
      repeated(6),
    );
    assert.deepEqual(
      ids(
        `
          import { left } from "./mod";
          function pick(flag, param) {
            const both = flag ? param : left;
            both.a = 1;
          }
        `,
        { parameters: false },
      ),
      ["mutation"],
    );
  });

  it("reports a shared or escaped object through ||, ??, &&, and ?:", () => {
    assert.deepEqual(
      ids(`
        function reassign(opts) {
          opts = opts || {};
          opts.x = 1;
          return opts;
        }
        function nullish(opts) {
          const o = opts ?? {};
          o.x = 1;
          return o;
        }
        function grouped(p) {
          (p || {}).x = 1;
        }
        function assigned(p) {
          Object.assign(p || {}, { a: 1 });
        }
        const mod = {};
        function choose(c) {
          const x = c ? mod : {};
          x.a = 1;
        }
        let g;
        function storeConditional(c) {
          const b = {};
          g = c ? b : null;
          b.a = 1;
        }
        function storeNullLeft(c) {
          const b = {};
          g = c ? null : b;
          b.a = 1;
        }
        function storeOr(c) {
          const b = {};
          g = b || null;
          b.a = 1;
        }
        function storeAnd(c) {
          const b = {};
          g = c && b;
          b.a = 1;
        }
        function storeNullish(c) {
          const b = {};
          g = b ?? 0;
          b.a = 1;
        }
        function storeArray(c) {
          const b = {};
          g = [c ? b : 0];
          b.a = 1;
        }
        function sinkJoin(c, sink) {
          const b = {};
          const x = c ? b : null;
          sink(x);
          b.a = 1;
        }
        function sinkOr(c, sink) {
          const b = {};
          const x = b || {};
          sink(x);
          b.a = 1;
        }
        let h;
        function storeClosure(c) {
          const b = {};
          const fn = () => {
            b.a = 1;
          };
          h = c ? fn : null;
          return b;
        }
        function callEither(c, sink) {
          const b = {};
          const left = () => {
            b.a = 1;
          };
          const right = () => {
            b.c = 2;
          };
          sink(b);
          const fn = c ? left : right;
          fn();
        }
        function fieldFns(c) {
          const b = {};
          const left = () => {
            b.a = 1;
          };
          const right = () => {
            b.c = 2;
          };
          const box = { slot: c ? left : right };
          g = box;
        }
        function nestedFresh(c, sink) {
          const b = {};
          const other = {};
          const box = {};
          box.slot = c ? b : other;
          sink(box);
          b.a = 1;
        }
        function carried(c, sink) {
          const b = {};
          let y = {};
          for (let i = 0; i < 2; i++) {
            y.a = 1;
            y = c ? y : b;
            sink(b);
          }
        }
        function readShared(c, param) {
          const a = { client: param };
          const b = { client: {} };
          const o = c ? a : b;
          o.client.id = 1;
        }
        function closureJoin(c) {
          const b = {};
          const other = {};
          const o = c ? b : other;
          return function inner() {
            o.a = 1;
          };
        }
        function outerSlot(c) {
          let slot;
          function inner() {
            const b = {};
            slot = c ? b : null;
            b.a = 1;
          }
          return inner;
        }
      `),
      repeated(23),
    );
  });

  it("keeps an unescaped join of fresh objects quiet", () => {
    assert.deepEqual(
      ids(`
        function local(c) {
          const b = {};
          const other = {};
          const key = "id";
          const peek = b[key];
          const holder = { nested: {} };
          const found = holder[key];
          const pair = [c ? b : other];
          void peek;
          void found;
          void pair;
          const o = c ? b : {};
          o.a = 1;
          const dup = (c ? b : o) || b;
          dup.a = 2;
          const viaMap = c ? new Map() : b;
          viaMap.a = 3;
          return o;
        }
        function sameLet(c) {
          let inner;
          const b = {};
          inner = c ? b : null;
          b.a = 1;
          const fn = () => {
            b.c = 2;
          };
          inner = c ? fn : null;
          return b;
        }
        function elements(c) {
          const a = [{ id: 1 }];
          const b = [{ id: 2 }];
          const [item] = c ? a : b;
          item.id = 3;
        }
        function members(c) {
          const a = { client: {} };
          const b = { client: {} };
          const o = c ? a : b;
          o.client.get = 1;
        }
        function quietCarry(c) {
          let y = {};
          const z = {};
          for (let i = 0; i < 2; i++) {
            y.a = 1;
            y = c ? y : z;
          }
        }
        let g;
        function early(c) {
          const b = {};
          b.a = 1;
          g = c ? b : null;
        }
        function captureDup(c, sink) {
          const b = {};
          const other = {};
          const joined = (c ? b : other) || b;
          return () => sink(joined);
        }
        class Box {
          write(flag) {
            const self = flag ? this : this;
            self.a = 1;
            const cache = new Map();
            const viaThis = flag ? this : cache;
            viaThis.a = 1;
            const viaMap = flag ? cache : this;
            viaMap.a = 1;
            const literal = { "id": 1, 2: flag };
            literal.extra = 2;
          }
        }
      `),
      [],
    );
  });

  it("allows Map and Set, including module reassignment", () => {
    assert.deepEqual(
      ids(`
        const cache = new Map();
        cache.set("a", 1);
        cache.extra = 2;
        const seen = new Set();
        seen.add(1);
        let replaced = {};
        replaced = new Map();
        replaced.a = 1;
        function shadowed(Map, Set, box) {
          const local = new Map();
          local.a = 1;
          box.a = 1;
        }
      `),
      ["mutation"],
    );
  });

  it("reports mutating calls and skips delete, shadowed Object, and computed names", () => {
    assert.deepEqual(
      ids(`
        const tables = [];
        tables.push(1);
        tables.pop();
        tables.shift();
        tables.unshift(1);
        tables.splice(0, 1);
        tables.sort();
        tables.reverse();
        tables.fill(0);
        tables.copyWithin(0, 1);
        tables["push"]();
        Object.assign(tables, {});
        Object.defineProperty(tables, "a", {});
        Object.defineProperties(tables, {});
        Object.setPrototypeOf(tables, {});
        (Object).assign(tables, {});
        Object.assign();
        Object.assign(...tables);
        delete tables[0];
        function shadowed(Object, rows) {
          Object.assign(rows, {});
        }
        function literal(push, rows) {
          rows.push(1);
        }
      `),
      repeated(16),
    );
  });

  it("turns calls, module, imported, and parameters off independently", () => {
    const code = `
      import { rows } from "./mod";
      const tables = {};
      tables.a = 1;
      tables.push(1);
      rows.a = 1;
      function write(box) {
        box.a = 1;
        box.push(1);
      }
    `;
    assert.deepEqual(ids(code, { calls: false }), repeated(3));
    assert.deepEqual(ids(code, { module: false }), repeated(3));
    assert.deepEqual(ids(code, { imported: false }), repeated(4));
    assert.deepEqual(ids(code, { parameters: false }), repeated(3));
  });

  it("honors allow names and paths, including static computed keys", () => {
    const code = `
      const tables = {};
      tables.users.id = 1;
      tables.users = 1;
      tables.orders = 1;
      tables[\`users\`] = 1;
      tables[key] = 1;
      function write(box) { box.a = 1; }
    `;
    assert.deepEqual(ids(code, { allow: ["tables"] }), ["mutation"]);
    assert.deepEqual(ids(code, { allow: ["tables.users"] }), repeated(3));
  });

  it("allows this in a class and reports it when allowThis is false", () => {
    const code = `
      function outside() { this.a = 1; }
      const object = { m() { this.a = 1; } };
      class Box extends Base {
        m() { this.a = 1; }
        static { this.b = 2; }
        arrow() {
          const write = () => { this.c = 3; };
          write();
        }
      }
      class Proto { }
      Proto.ready = true;
    `;
    assert.deepEqual(ids(code), ["mutation"]);
    assert.deepEqual(ids(code, { allowThis: false }), repeated(4));
  });

  it("supports both suppression styles", () => {
    assert.deepEqual(
      ids(`
        const tables = {};
        // eslint-disable-next-line no-mistakes/no-object-mutation -- third-party cache
        tables.a = 1;
        // no-mistakes-disable-next-line no-object-mutation -- third-party cache
        tables.b = 1;
        tables.c = 1; // no-mistakes-disable-line no-object-mutation -- third-party cache
        tables.d = 1;
      `),
      ["mutation"],
    );
    assert.deepEqual(
      ids(`
        /* no-mistakes-disable-file no-object-mutation */
        const tables = {};
        tables.a = 1;
      `),
      [],
    );
  });

  it("walks syntax that does not hide a shared write", () => {
    const reported = ids(
      `
        import def, { named, type TypeOnly } from "./mod";
        import * as ns from "./mod";
        export * from "./mod";
        export { named as alias, missing };
        export default function () { return 1; }
        export class Exported {}
        var moduleRows = [];
        moduleRows.push(1);
        function dec(value) { return value; }
        @dec
        class Box extends Base {
          @dec field = 1;
          #hidden = 1;
          [key]() { this.#hidden = 2; }
          constructor(public box) { super(); this.box = box; }
          static { this.ready = true; }
        }
        async function walk(flag, param, sink, rows, tag) {
          const box = { client: {}, get a() { return 1; }, set b(value) {}, ...param, [key]: 1 };
          const holes = [1, , ...rows];
          if (flag) sink(box);
          else return box;
          switch (flag) { case 1: break; default: break; }
          label: for (;;) { break label; }
          for (let i = 0; i < 1; i++) {}
          for (const key in param) {}
          while (flag) {}
          do {} while (flag);
          try { throw box; } catch { void 0; } finally { debugger; }
          function* gen() { yield; yield box; }
          await flag;
          tag\`\${box}\`;
          const text = \`\${box}\`;
          import(flag);
          import.meta.hot = 1;
          new.target;
          (rows).a++;
          typeof rows;
          (0, rows.a = 1);
          flag ? rows.b = 1 : 0;
          rows.c &&= 1;
          rows.d ||= 1;
          rows.e ??= 1;
          const chosen = flag ? named : ns;
          chosen.a = 1;
          const mixed = flag ? param : { a: 1 };
          mixed.a = 1;
          const maps = flag ? new Map() : new Map();
          maps.a = 1;
          const self = {};
          self.self = self;
          sink(self);
          let copy;
          ({ ...copy } = param);
          let first;
          [first, ...copy] = rows;
          ({ a: rows.z = 1 } = rows);
          const { ...rest } = param;
          const [...all] = rows;
          const view = <div attr={} {...rows}>{rows.y = 1}{...rows}</div>;
          const frag = <>{null}{ }</>;
          const typed = (rows as T).w = 1;
          const satisfied = rows satisfies T;
          const present = rows!;
          const inner = class Local {};
          function pushName(push, items) { items["push"](); }
          undefined = 1;
          named = 1;
          sink(notDeclared);
          sink(walk);
        }
      `,
      undefined,
      "fixture.tsx",
    );
    assert.ok(reported.length > 5);
    assert.ok(reported.every((id) => id === "mutation"));
  });

  it("walks patterns, joins, and expression bodies", () => {
    const reported = ids(`
      import { left, right } from "./mod";
      export function hoistedExport() {
        return 1;
      }
      function exported() {}
      export { exported };
      export default function namedDefault(box) {
        box.a = 1;
      }
      let key = "id";
      function patterns(rows, sink, param, flag) {
        for (param.item of rows) {}
        for (param.name in rows) {}
        for (const item of rows) item.a = 1;
        for (const entry of new Map()) entry.a = 1;
        for (const value of this) value.a = 1;
        for (const mystery of sink()) mystery.a = 1;
        try {
          sink(1);
        } finally {
          void 0;
        }
        try {
          sink(2);
        } catch (error) {
          void error;
        }
        return;
        const write = (box) => (box.a = 1, box);
        write(param);
        write(param);
        const box = {};
        box[key] = param;
        ++box[key];
        ++box.a;
        function poke() {
          box.b = 1;
        }
        poke();
        poke();
        sink([param]);
        new Map([["a", param]]);
        new Wrapper(param);
        const { nested, [key]: computed, ...rest } = param;
        const [item, , extra] = rows;
        ({ [key]: param.z } = param);
        [param.y, , param.w] = rows;
        const sameSpread = { ...left, ...left };
        const both = { ...left, ...right };
        const caches = [new Map(), new Map()];
        const dynamic = box[key];
        sink(box);
        const after = box.client;
        after.a = 1;
        const chain = param?.client;
        chain.id = 1;
        (param).ready = true;
        sink().slot = 1;
        const self = flag ? this : this;
        self.a = 1;
        const mixedThis = flag ? this : param;
        mixedThis.a = 1;
        const otherThis = flag ? param : this;
        otherThis.a = 1;
        if (flag) {
          function late(box) {
            box.a = 1;
          }
        }
      }
      function take({ a, [key]: b, ...rest }, [first, , ...tail]) {}
      class BareField {
        unset;
        [key];
      }
      const sharedBox = {};
      function repeat(xs, sink) {
        const box = {};
        for (const item of xs) {
          sharedBox.a = 1;
          sink(box);
        }
      }
    `);
    assert.ok(reported.length > 8);
    assert.ok(reported.every((id) => id === "mutation"));
    assert.deepEqual(ids("function write(rows) {\n  (rows).a = 1;\n}\n", undefined, "fixture.js"), [
      "mutation",
    ]);
  });

  it("tracks fresh elements, mixed joins, and non-member allow paths", () => {
    assert.deepEqual(
      ids(
        `
          import { row } from "./mod";
          function build(sink, rows, flag, key) {
            const local = [{ id: 1 }, row];
            const [first] = local;
            first.id = 2;
            const empty = [];
            const [missing] = empty;
            missing.a = 1;
            sink(local);
            const [escaped] = local;
            escaped.id = 3;
            const box = {};
            const client = box.missing;
            client.a = 1;
            const mixed = { ...new Map(), ...row };
            mixed.a = 1;
            rows?.push(1);
            (Object as T).assign(rows, { a: 1 });
            (flag ? rows : rows).id = 1;
            rows[key]();
            for (const item of [{ id: 1 }]) item.id = 2;
            for (const item of empty) item.id = 4;
          }
        `,
        { allow: ["rows.keep"] },
      ),
      repeated(4),
    );
  });

  it("reports through oxlint jsPlugins", () => {
    const root = mkdtempSync(join(tmpdir(), "no-object-mutation-oxlint-"));
    try {
      writeFileSync(join(root, "fixture.js"), "const tables = {};\ntables.a = 1;\n");
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: { "no-mistakes/no-object-mutation": "error" },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.js",
        ],
        { cwd: root, encoding: "utf8" },
      );
      assert.notEqual(result.status, 0);
      assert.match(`${result.stdout || ""}${result.stderr || ""}`, /statically/);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
