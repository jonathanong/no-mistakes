import assert from "node:assert/strict";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { messages, require } from "./helpers.mjs";

const { isUnresolved, moduleLoad } = require("../src/rules/module-load.js");
const { matchDirectMockCallApply } = require("../src/rules/module-mock-call-apply.js");
const { frameworkBindingOpaque } = require("../src/rules/module-mock-framework.js");
const { forwardedCallee } = require("../src/rules/no-vitest-sequential-require.js");
const {
  hasProperty,
  hidesComputedKey,
  resolveVariable,
} = require("../src/rules/test-no-shared-state-aliases.js");
const {
  replayPendingCallbacks,
  reportOpaqueRegistration,
} = require("../src/rules/test-no-shared-state-imports.js");
const { contextParameter, testBinding } = require("../src/rules/test-no-skips-bindings.js");
const { requireExport, requireMemberBinding } = require("../src/rules/test-no-skips-members.js");
const { opaqueFrameworkReceiver } = require("../src/rules/test-no-typed-enqueue-fake-opaque.js");
const { moduleMember } = require("../src/rules/test-no-wall-time-assertions-bindings.js");

function withContext(code, visitor, filename = "probe.test.ts") {
  const linter = new Linter({ configType: "flat", cwd: "/tmp" });
  linter.verify(
    code,
    {
      files: ["**"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
        parser: tsParser,
      },
      plugins: {
        local: {
          rules: {
            probe: {
              create(context) {
                return visitor(context);
              },
            },
          },
        },
      },
      rules: { "local/probe": "error" },
    },
    filename,
  );
}

describe("CommonJS and TypeScript wrapper edge coverage", () => {
  it("covers moduleLoad await, named-export destructure, and literal import edges", () => {
    const loads = new Map();
    withContext(
      `
      import { "vi" as litVi, test as namedTest } from "vitest";
      const { nested: { inner } } = require("vitest");
      const { ["vi"]: computedVi, ...rest } = require("vitest");
      const { sub } = namedTest;
      async function run() {
        const awaitedPlain = await foo();
        const awaitedReq = await require("vitest");
        return [litVi, inner, computedVi, rest, sub, awaitedPlain, awaitedReq];
      }
      `,
      (context) => ({
        Identifier(node) {
          if (
            ["litVi", "inner", "computedVi", "rest", "sub", "awaitedPlain", "awaitedReq"].includes(
              node.name,
            )
          ) {
            loads.set(node.name, moduleLoad(node, context));
          }
        },
      }),
    );
    assert.deepEqual(loads.get("litVi"), { source: "vitest", exportName: "vi" });
    assert.equal(loads.get("inner"), null);
    assert.deepEqual(loads.get("computedVi"), { source: "vitest", exportName: "vi" });
    assert.equal(loads.get("rest"), null);
    assert.equal(loads.get("sub"), null);
    assert.equal(loads.get("awaitedPlain"), null);
    assert.deepEqual(loads.get("awaitedReq"), { source: "vitest", exportName: "*" });
    assert.equal(isUnresolved(null), false);
    assert.equal(moduleLoad(null, {}), null);
  });

  it("covers module-mock call/apply spread and sparse array edges", () => {
    assert.equal(
      messages(
        `
        declare const mod: string;
        const dyn = require(mod);
        const args = ["./a", () => ({})];
        vi.mock.call(...args);
        vi.mock.apply(vi, [, () => ({})]);
        vi.mock.apply(vi, [...args]);
        dyn.mock.call(...args);
        dyn.mock.apply(dyn, [...args]);
        `,
        "module-mock-boundary",
        undefined,
        "mock.test.ts",
      ).includes("unanalyzable"),
      true,
    );

    assert.equal(
      messages(
        `
        import * as V from "vitest";
        declare const dynKey: string;
        declare const mod: string;
        const { [dynKey]: hidden, ...rest } = require("vitest");
        const { vi: defaulted = fallback } = require("vitest");
        const outerDyn = require(mod);
        function inner() {
          outerDyn.mock("./x", () => ({}));
          hidden.mock("./x", () => ({}));
          rest.mock("./x", () => ({}));
          defaulted.mock("./x", () => ({}));
        }
        `,
        "module-mock-boundary",
        undefined,
        "mock.test.ts",
      ).includes("unanalyzable"),
      true,
    );

    const opaqueFlags = [];
    withContext(
      `
      import Local = Other.Ns;
      const fn = require(mod);
      function inside() {
        return [Local, fn, require(mod)];
      }
      `,
      (context) => ({
        "Identifier, CallExpression"(node) {
          if (node.type === "CallExpression" || node.name === "Local" || node.name === "fn") {
            opaqueFlags.push(frameworkBindingOpaque(node, context));
          }
        },
      }),
    );
    assert.equal(opaqueFlags.includes(true), true);
    assert.equal(
      matchDirectMockCallApply({ callee: { type: "Identifier", name: "mock" } }, {}, new Set()),
      null,
    );
  });

  it("covers typed enqueue opaque receiver and type-only import-equals helpers", () => {
    assert.deepEqual(
      messages(
        `
        declare const mod: string;
        let dyn = require(mod);
        dyn = other;
        dyn.vi.fn<() => void>();
        let { vi } = require(mod);
        vi = other;
        vi.fn<() => void>();
        function run(param: any) {
          param.vi.fn<() => void>();
          getObj().vi.fn<() => void>();
          require("other").vi.fn<() => void>();
        }
        `,
        "test-no-typed-enqueue-fake",
        { singleExportPattern: "^enqueue$" },
        "enqueue.test.ts",
      ),
      [],
    );

    assert.deepEqual(
      messages(
        `
        import type V = require("vitest");
        import Local = Other.Ns;
        import { enqueue } from "./queue";
        export const a: V.Mock<typeof enqueue> = vi.fn();
        export const b: Local.Mock<typeof enqueue> = other();
        `,
        "test-no-typed-enqueue-fake",
        { singleExportPattern: "^enqueue$" },
        "enqueue-types.test.ts",
      ),
      ["typedEnqueueFake"],
    );

    const results = [];
    withContext(
      `
      const x = require("vitest");
      x.vi;
      `,
      (context) => ({
        MemberExpression(node) {
          results.push(opaqueFrameworkReceiver(node, context.sourceCode));
        },
      }),
    );
    assert.deepEqual(results, [false]);
  });

  it("covers shared-state alias, import, and replay edge branches", () => {
    assert.equal(
      hasProperty(
        {
          type: "SequenceExpression",
          expressions: [
            { type: "Literal", value: 0 },
            {
              type: "ChainExpression",
              expression: {
                type: "MemberExpression",
                computed: false,
                object: { type: "Identifier", name: "test" },
                property: { type: "Identifier", name: "describe" },
              },
            },
          ],
        },
        "describe",
      ),
      true,
    );
    assert.equal(hidesComputedKey(null, {}), false);

    assert.deepEqual(
      messages(
        `
        let t = require("vitest").test;
        t = other;
        let plain = other;
        plain.test("a", () => {});
        let { sub } = require(mod);
        sub.test("b", () => {});
        const d = (0, describe);
        d?.("suite", () => {});
        `,
        "test-no-shared-state",
        undefined,
        "shared.test.ts",
      ),
      ["unanalyzable"],
    );

    let flushed = false;
    replayPendingCallbacks({
      depth: { setup: 0, test: 0 },
      cleanupTracker: {
        beginSetup() {},
        endSetup() {},
        setReplaySuite() {},
        clearReplaySuite() {},
      },
      mutationHandlers: { mutationWalk() {} },
      registryReports: {
        flush() {
          flushed = true;
        },
      },
      walkSharedMutations() {},
      pendingNamedSetupCallbacks: [{ declaration: null, suiteKey: "s", kind: "beforeEach" }],
      pendingNamedCallbacks: [{ declaration: null, suiteKey: "s" }],
    });
    assert.equal(flushed, true);

    withContext(
      `
      const dyn = require(mod);
      dyn.test("a", () => {});
      `,
      (context) => ({
        CallExpression(node) {
          if (node.callee.type === "MemberExpression") {
            reportOpaqueRegistration({ ...node, loc: null }, context, () => true, node);
            assert.ok(resolveVariable(node.callee.object, context));
          }
        },
      }),
    );
  });

  it("covers skips bindings, members, and wall-time bindings edges", () => {
    assert.equal(testBinding(null, {}), null);
    assert.equal(contextParameter(null, {}), null);
    assert.equal(requireExport(null, {}), null);
    assert.equal(
      requireMemberBinding(null, null, {}, () => null, new Set()),
      null,
    );
    assert.equal(forwardedCallee(null), null);

    assert.deepEqual(
      messages(
        `
        const [first] = require("vitest");
        const dyn = require("vitest")[key];
        first.skip("a", () => {});
        dyn.skip("b", () => {});
        `,
        "test-no-skips",
        undefined,
        "skips.test.ts",
      ),
      [],
    );

    assert.deepEqual(
      messages(
        `
        const { ["performance"]: perf } = require("node:perf_hooks");
        const start = perf.now();
        expect(perf.now() - start).toBeLessThan(100);
        `,
        "test-no-wall-time-assertions",
        undefined,
        "wall.test.ts",
      ),
      ["wallTime"],
    );

    const wallResults = [];
    withContext(
      `
      var dup = require("node:perf_hooks");
      var dup = other;
      const loop = loop;
      dup;
      loop.now();
      `,
      (context) => ({
        Identifier(node) {
          if (node.name === "dup" || node.name === "loop") {
            wallResults.push(moduleMember(node, context));
          }
        },
      }),
    );
    assert.equal(
      wallResults.every((entry) => entry === null),
      true,
    );
  });

  it("covers vitest-timeout-cap resolver, callback bindings, temp-paths, and error-message edges", () => {
    assert.deepEqual(
      messages(
        `
        import { test, vi } from "vitest";
        declare const mod: string;
        declare const dyn: string;
        const other = require("other").foo();
        const req = require(mod);
        const boundReq = req.foo.bind(other);
        const boundTest = test.bind(other);
        const boundSetter = vi.setConfig.bind(vi, extra);
        const a = b;
        const b = a;
        function fn() {}
        fn = other;
        const dynKey = "ti" + dyn;
        test("x", a, 100);
        test("y", fn, 100);
        test[dynKey]("z", () => {}, 100);
        boundReq("a", () => {}, 100);
        boundTest("b", () => {}, 100);
        boundSetter({ testTimeout: 100 });
        test("ctx", ({ onTestFinished }) => {
          onTestFinished(() => {}, 99999);
        });
        other;
        `,
        "vitest-timeout-cap",
        { unknownValues: "finding" },
        "timeout.test.ts",
      ).includes("timeout"),
      true,
    );

    assert.deepEqual(
      messages(
        `
        import type fsType = require("node:fs");
        import fsLocal = Local.Fs;
        declare const mod: string;
        const [arrFs] = require(mod);
        arrFs("/tmp/fixed-a");
        fsLocal.writeFileSync("/tmp/fixed-b", "x");
        export type T = typeof fsType;
        `,
        "test-no-fixed-temp-paths",
        undefined,
        "temp.test.ts",
      ),
      ["unanalyzable"],
    );

    assert.deepEqual(
      messages(
        `
        declare const mod: string;
        const dyn = require(mod);
        declare const rest: unknown[];
        dyn.expect.apply(null, [, err.message]).toBe("x");
        dyn.expect(...[, err.message]).toBe("x");
        dyn.expect(...[...rest]).toBe("x");
        dyn.expect(err.message, ...[...rest]).toBe("x");
        `,
        "test-no-error-message-matching",
        undefined,
        "msg.test.ts",
      ),
      ["unanalyzable"],
    );
  });
});
