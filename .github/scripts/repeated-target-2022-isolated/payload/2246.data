import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
import {
  BaseRunner,
  withBothFields,
} from "../../../fixtures/test-timeout-policy/derived-fields/runners.mjs";
import { guardPublicRunner, guardRunnerStatics } from "./public-runner.mjs";
import { guardVitestExports, protectRunnerTimeouts } from "./index.mjs";

describe("public runner static registration", () => {
  it.skipIf(process.platform !== "linux")(
    "proves native raw static hooks lack captured deadlines",
    async () => {
      const source = readFileSync(
        new URL(
          "../../../fixtures/test-timeout-policy/runner-statics/native-raw-hook.txt",
          import.meta.url,
        ),
        "utf8",
      );
      const result = await fixture(source);
      expect(result.code, result.output).toBe(0);
      expect(result.extensionMarker).toBe("raw callback entered");
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
      expect(result.signal).toBeNull();
    },
    10000,
  );

  it.skipIf(process.platform !== "linux")(
    "guards actual Node SDK static registrations and preserves unsupported sequential behavior",
    async () => {
      const source = readFileSync(
        new URL(
          "../../../fixtures/test-timeout-policy/runner-statics/public-sdk.txt",
          import.meta.url,
        ),
        "utf8",
      );
      const result = await fixture(source);
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );

  it.each(["node", "browser"])(
    "guards collector statics accessed directly or through derived %s constructors",
    (mode) => {
      const rawHook = vi.fn();
      const test = vi.fn();
      const collector = { on: rawHook, test };
      class Native extends BaseRunner {
        static getCurrentSuite = () => collector;
        static createTaskCollector = () => test;
        static getSuiteHooks = vi.fn();
        static setSuiteHooks = Native.getSuiteHooks;
        static unchanged = () => "public alias";
      }
      const Guarded =
        mode === "node"
          ? guardRunnerStatics(Native, 25, protectRunnerTimeouts)
          : guardVitestExports({ TestRunner: Native }, 25).TestRunner;
      class Derived extends Guarded {}
      for (const Constructor of [Guarded, Derived]) {
        const result = Constructor.getCurrentSuite();
        expect(result).toBe(Constructor.getCurrentSuite());
        expect(() => result.on("beforeAll", () => new Promise(() => {}))).toThrow(
          "bypasses native hook deadlines",
        );
        expect(() => result.test("invalid", { timeout: 26 }, () => {})).toThrow(RangeError);
        expect(() => Constructor.createTaskCollector(() => {})("invalid", () => {}, 0)).toThrow(
          RangeError,
        );
        for (const name of ["getSuiteHooks", "setSuiteHooks"])
          expect(() => Constructor[name]({})).toThrow(
            "raw suite hooks bypass native hook deadlines",
          );
        expect(Constructor.unchanged).toBe(Native.unchanged);
        Constructor.createTaskCollector(() => {})("valid", () => {}, 25);
      }
      expect(rawHook).not.toHaveBeenCalled();
      expect(Native.getSuiteHooks).not.toHaveBeenCalled();
      expect(test).toHaveBeenCalledTimes(2);
    },
  );

  it("keeps Node base construction unprotected until the final constructor fields exist", () => {
    const protect = vi.fn(protectRunnerTimeouts);
    const Facade = guardRunnerStatics(BaseRunner, 25, protect);
    const direct = new Facade({ testTimeout: 25, hookTimeout: 25 });
    expect(direct.target).toBe(BaseRunner);
    expect(protect).not.toHaveBeenCalled();
    const Derived = withBothFields(Facade);
    const Final = guardPublicRunner(Derived, 25, protect);
    const runner = new Final({ testTimeout: 25, hookTimeout: 25 });
    expect(runner.target).toBe(Derived);
    expect(runner.onBeforeCollect([])).toEqual(["base", "collection"]);
    const context = { task: { timeout: 25 } };
    expect(runner.extendTaskContext(context).identities).toEqual(["base", "derived"]);
    expect(protect).toHaveBeenCalledOnce();
    expect(() => runner.extendTaskContext({ task: { timeout: 26 } })).toThrow(RangeError);
  });
});
