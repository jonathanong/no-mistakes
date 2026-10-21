import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";

const RULE = "no-object-mutation";

function ids(code, option, filename = "fixture.ts") {
  return messages(code, RULE, option, filename);
}

function repeated(count) {
  return Array.from({ length: count }, () => "mutation");
}

describe("no-object-mutation escapes", () => {
  it("ignores an ambient module that has no body", () => {
    assert.deepEqual(
      ids(`declare module "*.svg";\ndeclare module "foo";`, undefined, "types.d.ts"),
      [],
    );
  });

  it("reports arguments, aliases, and not a shadowed local builder", () => {
    assert.deepEqual(
      ids(`
        function write(box) {
          arguments[0].b = 1;
          const alias = arguments;
          alias[0].c = 1;
          const nested = () => {
            arguments[0].d = 1;
          };
          nested();
        }
        function shadowed() {
          const arguments = {};
          arguments.a = 1;
          return arguments;
        }
        const arrow = () => {
          arguments[0].b = 1;
        };
        function parameter(arguments) {
          arguments[0].b = 1;
        }
      `),
      repeated(4),
    );
    assert.deepEqual(
      ids(
        `
          function write(box) {
            arguments[0].b = 1;
          }
        `,
        { parameters: false },
      ),
      [],
    );
  });

  it("reports Reflect and forwarded array writes, not a local builder", () => {
    assert.deepEqual(
      ids(`
        function write(box, rows) {
          Reflect.set(box, "a", 1);
          Reflect.defineProperty(box, "b", { value: 1 });
          Array.prototype.push.call(rows, 1);
          [].push.apply(rows, [2]);
          rows.push.call(box, 3);
          Object.assign.call(null, box, { c: 1 });
          Object.assign.apply(null, [box, { c: 1 }]);
          Object.assign.bind(null)(box, { c: 1 });
          Object.assign.call(box, { c: 1 });
          Object.assign.apply(box, [{ c: 1 }]);
        }
        function local() {
          const box = {};
          const rows = [];
          Reflect.set(box, "a", 1);
          Array.prototype.push.call(rows, 1);
          [].push.apply(rows, [2]);
          return { box, rows };
        }
        function shadowed(Reflect, Object, rows) {
          Reflect.set(rows, "a", 1);
          Object.assign(rows, {});
          Object.assign.bind(null)(rows, {});
        }
        function missing(box) {
          Reflect.set();
          Reflect.set(...box);
          Array.prototype.push.call();
          Array.prototype.push.apply();
          box.unknown.call(box, 1);
          Object.assign.call(box);
          Object.assign.apply(box, ...rows);
          Object.assign.apply(box, [...rows]);
          Object.assign.apply(null, []);
          Object.assign.bind(null)();
          Object.assign.bind(null)(...box);
          bind(null)(box);
          Object.assign.call(null)(box, { c: 1 });
          fn.bind(null)(box);
          box.push.bind(box)(1);
          box[missingName].bind(null)(box);
        }
      `),
      repeated(8),
    );
    assert.deepEqual(
      ids(
        `
          function write(box, rows) {
            Reflect.set(box, "a", 1);
            Array.prototype.push.call(rows, 1);
          }
        `,
        { calls: false },
      ),
      [],
    );
  });

  it("reports map reads and Object.values only when the receiver is shared", () => {
    assert.deepEqual(
      ids(`
        const cache = new Map();
        cache.set("a", 1);
        cache.get(1).a = 1;
        for (const value of Object.values(cache)) value.x = 1;
        for (const value of cache) value.y = 1;
        const tables = { row: {} };
        for (const row of Object.values(tables)) row.ready = true;
        function fromParam(map, box) {
          map.get(1).a = 1;
          for (const value of Object.values(box)) value.a = 1;
        }
        function local() {
          const map = new Map();
          const box = {};
          map.get(1).a = 1;
          for (const value of Object.values(box)) value.b = 1;
          for (const value of map) value.c = 1;
          const fresh = { nested: {} };
          for (const value of Object.values(fresh)) value.d = 1;
          return fresh;
        }
        function valuesOfShared(flag, left, right) {
          const map = flag ? left : right;
          for (const value of Object.values(map)) value.a = 1;
        }
        function notValues(Object, box) {
          for (const value of Object.values(box)) value.a = 1;
        }
        function noTarget(box) {
          Object.values();
          Object.values(...box);
        }
      `),
      repeated(7),
    );
  });

  it("reports namespace exports and keeps a local namespace builder", () => {
    assert.deepEqual(
      ids(`
        namespace Registry {
          export const rows = {};
          const local = {};
          local.a = 1;
          export function write(box) {
            box.a = 1;
          }
          export { local as escaped };
        }
        Registry.rows.users = "app.users";
        namespace Registry {
          export const more = {};
        }
        Registry.more.n = 1;
        namespace Outer.Inner {
          export const cell = {};
        }
        Outer.Inner.cell.a = 1;
        namespace Outer {
          namespace Hidden {
            export const rows = {};
          }
          Hidden.rows.x = 1;
        }
        declare module "fs" {
          export const path = {};
        }
        function localNamespace() {
          const box = {};
          namespace Local {
            export const rows = box;
          }
          Local.rows.a = 1;
          return box;
        }
      `),
      repeated(4),
    );
  });

  it("reports this in plain functions and escaped object methods", () => {
    assert.deepEqual(
      ids(`
        function publish() {
          this.client = null;
        }
        const api = {
          save() {
            this.dirty = true;
          },
        };
        function local() {
          const box = {
            save(value) {
              this.current = value;
            },
          };
          box.save(1);
          return box.current;
        }
        function escaped(sink) {
          const box = {
            save(value) {
              this.current = value;
            },
          };
          sink(box);
        }
        const later = {};
        const writer = {
          save() {
            later.a = 1;
          },
        };
      `),
      repeated(4),
    );
  });

  it("runs a plain new and keeps field this on the instance", () => {
    assert.deepEqual(
      ids(`
        function plainNew(box) {
          function Writer() {
            box.a = 1;
          }
          new Writer();
        }
        function plainNewLocal() {
          const box = {};
          function Writer() {
            box.a = 1;
          }
          new Writer();
          return box;
        }
        function fieldThis() {
          const box = {
            save() {
              return class {
                field = (this.ready = true);
              };
            },
          };
          return box.save();
        }
      `),
      ["mutation"],
    );
  });

  it("reports nested this, called methods, and thisArg side effects", () => {
    assert.deepEqual(
      ids(`
        function nested(box) {
          class Local {
            m() {
              function mutate() {
                this.x = 1;
              }
              mutate();
              this.y = box;
            }
          }
          new Local().m();
        }
        function called(box) {
          class Writer {
            m() {
              box.a = 1;
            }
          }
          sink(box);
          new Writer().m();
        }
        function side(param) {
          Object.assign.call((param.x = 1), {}, {});
          Object.assign.apply((param.y = 1), [{}, {}]);
        }
      `),
      repeated(4),
    );
  });

  it("reports a class or bind that escapes and keeps a local class", () => {
    assert.deepEqual(
      ids(`
        function classExpr() {
          const box = {};
          return class {
            m() {
              box.a = 1;
            }
          };
        }
        function classValue() {
          const box = {};
          class Keeper {
            m() {
              box.a = 1;
            }
          }
          return Keeper;
        }
        function constructed() {
          const box = {};
          class Keeper {
            m() {
              box.a = 1;
            }
          }
          return new Keeper();
        }
        function bound(sink) {
          const box = {};
          function write() {
            box.a = 1;
          }
          return write.bind(null);
        }
        function either(flag) {
          const box = {};
          class Left {
            m() {
              box.a = 1;
            }
          }
          class Right {
            m() {
              box.b = 1;
            }
          }
          class Child extends (flag ? Left : Right) {}
          return new Child();
        }
        function self() {
          const box = {};
          class Loop extends Loop {
            m() {
              box.a = 1;
            }
          }
          return new Loop();
        }
        function localClass(value) {
          const box = {};
          class Writer {
            unset;
            field = 1;
            static {
              box.ready = true;
            }
            set() {
              box.current = value;
            }
          }
          const writer = new Writer();
          writer.set();
          return box;
        }
        function localBind() {
          const box = {};
          function write() {
            box.a = 1;
          }
          const boundWrite = write.bind(null);
          boundWrite();
          return box;
        }
        function named(value) {
          const box = {};
          return class Name {
            m() {
              box.a = Name;
              box.b = value;
            }
          };
        }
        function fieldLater() {
          const box = {};
          return class {
            field = (box.ready = true);
          };
        }
        function constructorLater() {
          const box = {};
          class Writer {
            constructor() {
              box.ready = true;
            }
          }
          return Writer;
        }
        function constructorRan() {
          const box = {};
          class Writer {
            constructor() {
              box.ready = true;
            }
          }
          return new Writer();
        }
        function staticLater() {
          const box = {};
          class Writer {
            static save() {
              box.ready = true;
            }
          }
          return Writer;
        }
        function staticRan() {
          const box = {};
          class Writer {
            static save() {
              box.ready = true;
            }
          }
          return new Writer();
        }
      `),
      repeated(12),
    );
    assert.deepEqual(
      ids(`
        function joins(flag, param, key, sink) {
          Object.values(param, flag);
          Object.values(param, ...key);
          function write() {
            param.a = 1;
          }
          function other() {
            param.b = 1;
          }
          sink(write.bind(...key));
          sink((flag ? write : other).bind(null));
          const holder = { get: function read() {} };
          holder.get(1);
          sink().get(1);
          const map = flag ? param : new Map();
          map.get(1).c = 1;
          const plain = {};
          const box = flag ? param : plain;
          box.get(1);
          this.get(1).d = 1;
          for (const value of Object.values(sink())) value.e = 1;
          for (const value of Object.values(write)) value.f = 1;
          const tables = flag ? param : { row: {}, method() {}, [key]: param };
          for (const value of Object.values(tables)) value.g = 1;
          const local = { a: {} };
          const { a = {} } = local;
          a.h = 1;
          write.call(param, 1);
          param[key].call(param, 1);
          class Left {
            m() {
              param.i = 1;
            }
          }
          class Right {
            m() {
              param.j = 1;
            }
          }
          return new (flag ? Left : Right)();
        }
        const cache = new Map();
        function publish(sink) {
          sink(cache);
        }
        namespace Outer {
          export namespace Inner {
            export const cell = {};
            export const bare;
            export function write() {
              cell.a = 1;
            }
          }
        }
        Outer.Inner.cell.b = 1;
        Outer.Inner.bare.c = 1;
      `),
      repeated(10),
    );
    assert.deepEqual(
      ids(`
        function dec(Class) {
          return Class;
        }
        function decorated() {
          const box = {};
          @dec
          class Writer {
            @dec
            m() {
              box.a = 1;
            }
          }
          return box;
        }
      `),
      ["mutation"],
    );
  });
});
