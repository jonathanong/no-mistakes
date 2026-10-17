import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
import { captureOwnedChild } from "../../../fixtures/test-timeout-policy/owned-sdk-child.mjs";
describe.skipIf(process.platform !== "linux")("supported Vitest registration matrix", () => {
  it.each([
    fixtureTemplate("registration-matrix.test-template-11.txt"),
    fixtureTemplate("registration-matrix.test-template-10.txt"),
    fixtureTemplate("registration-matrix.test-template-9.txt"),
    fixtureTemplate("registration-matrix.test-template-8.txt"),
    fixtureTemplate("registration-matrix.test-template-7.txt"),
    fixtureTemplate("registration-matrix.test-template-6.txt"),
    fixtureTemplate("registration-matrix.test-extra-template-5.txt"),
    fixtureTemplate("registration-matrix.test-extra-template-4.txt"),
    fixtureTemplate("registration-matrix.test-extra-template-3.txt"),
  ])("rejects native registration before a legal child can mask it: %s", async (source) => {
    const result = await fixture(source);
    expect(result.code).toBe(1);
    expect(result.signal).toBeNull();
    expect(result.output).toContain("timeout must be positive, finite and at most 30000 ms");
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["override", "scoped"])("preserves %s data and inherited context", async (method) => {
    const result = await fixture(
      fixtureTemplate("registration-matrix.test-template-5.txt", [method]),
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["each", "for"])(
    "preserves shuffled suite.%s with inclusive deadlines",
    async (method) => {
      const result = await fixture(
        fixtureTemplate("registration-matrix.test-template-4.txt", [method]),
      );
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it.each(["file", "worker"])("retains actual %s fixture setup and teardown", async (scope) => {
    const result = await fixture(
      fixtureTemplate("registration-matrix.test-template-3.txt", [scope]),
    );
    expect(result.code, result.output).toBe(0);
    expect(result.lifecycle).toBe("setup;body;body;teardown;");
    expect(result.forcedCleanup).toBe(false);
  });
  it("preserves native rejection of unsupported suite fixture scope", async () => {
    const result = await fixture(fixtureSource("registration-matrix.test-27.txt"));
    expect(result.code).toBe(1);
    expect(result.output).toContain('unknown scope "suite"');
    expect(result.marker).toBeUndefined();
  });
  it.each([false, true])(
    "routes a listed transitive registrar with symlinks=%s",
    async (preserveSymlinks) => {
      const result = await fixture(
        fixtureSource("registration-matrix.test-28.txt"),
        undefined,
        "",
        {
          preserveSymlinks,
          inlineTransitive: true,
          source: fixtureTemplate("registration-matrix.test-extra-template-2.txt"),
          transitiveSource: fixtureTemplate("registration-matrix.test-template-2.txt"),
        },
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("demonstrates an omitted externalized transitive registrar is an adoption gap", async () => {
    const result = await fixture(fixtureSource("registration-matrix.test-29.txt"), undefined, "", {
      preserveSymlinks: true,
      inlineTransitive: false,
      source: fixtureTemplate("registration-matrix.test-extra-template-1.txt"),
      transitiveSource: fixtureTemplate("registration-matrix.test-template-1.txt"),
    });
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
});
describe.skipIf(process.platform !== "linux")("started SDK child failure finalization", () => {
  it.each(["capture", "watchdog", "both"])(
    "awaits final output and drain after %s failure",
    async (failure) => {
      const original = new TypeError("injected ownership capture failure");
      let finalized = false;
      const run = fixture(
        fixtureSource("registration-matrix.test-30.txt"),
        undefined,
        "",
        undefined,
        {
          capture: (child) => {
            if (failure !== "watchdog") throw original;
            captureOwnedChild(child);
            return () => false;
          },
          watchdogMs: failure === "capture" ? 5000 : 1,
          closeGraceMs: 5000,
          onFinalized: (outcome) => {
            finalized = true;
            expect(outcome.closed).toBe(true);
            expect(outcome.drained).toBe(true);
            expect(outcome.result?.code).toBe(0);
            expect(outcome.output).toContain("1 passed");
            expect(outcome.forcedCleanup).toBe(false);
          },
        },
      );
      if (failure === "capture") await expect(run).rejects.toBe(original);
      else if (failure === "watchdog")
        await expect(run).rejects.toThrow("SDK deadline expired without a live owned group leader");
      else {
        const error = await run.catch((error) => error);
        expect(error).toBeInstanceOf(AggregateError);
        expect(error.cause).toBe(original);
        expect(error.errors[0]).toBe(original);
        expect(error.errors[1].message).toContain("SDK deadline expired");
      }
      expect(finalized).toBe(true);
    },
  );
});
