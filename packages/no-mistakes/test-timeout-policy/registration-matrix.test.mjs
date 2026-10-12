import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
import { captureOwnedChild } from "../../../fixtures/test-timeout-policy/owned-sdk-child.mjs";
describe("supported Vitest registration matrix", () => {
  it.each([
    `describe.shuffle('invalid suite', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `const shuffled = suite.shuffle; shuffled('invalid suite', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `describe.shuffle.each([1])('invalid %s', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `suite.shuffle.for([1])('invalid %s', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `describe.each([1])('invalid %s', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `suite.for([1])('invalid %s', { timeout: 30001 }, () => test('legal child', continued, 25))`,
    `test.for([1])('invalid %s', { timeout: 30001 }, continued)`,
    `const child = test.extend({ sample: 1 }).override({ sample: 2 }); child('invalid override', continued, 30001)`,
    `const child = test.extend({ sample: 1 }).scoped({ sample: 2 }); child('invalid scoped', continued, 30001)`,
  ])("rejects native registration before a legal child can mask it: %s", async (source) => {
    const result = await fixture(source);
    expect(result.code).toBe(1);
    expect(result.signal).toBeNull();
    expect(result.output).toContain("timeout must be positive, finite and at most 30000 ms");
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["override", "scoped"])("preserves %s data and inherited context", async (method) => {
    const result = await fixture(`
      const child = test.extend({ sample: { timeout: 60000 } })
        .${method}({ sample: { timeout: 60000, owned: true } })
      child.for([7])('legal %s', { timeout: 30000 }, (value, { sample, ownedRunnerMarker }) => {
        if (value !== 7 || !sample.owned || sample.timeout !== 60000)
          throw new Error('fixture override changed')
        if (ownedRunnerMarker !== 'preserved') throw new Error('context lost')
        continued()
      })
    `);
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["each", "for"])(
    "preserves shuffled suite.%s with inclusive deadlines",
    async (method) => {
      const result =
        await fixture(`suite.shuffle.${method}([7])('legal %s', { timeout: 30000 }, value => {
      if (value !== 7) throw new Error('suite table value changed')
      test('legal child', continued, 25)
    })`);
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it.each(["file", "worker"])("retains actual %s fixture setup and teardown", async (scope) => {
    const result = await fixture(`
      import { appendFileSync } from 'node:fs'
      const child = test.extend({ sample: [async ({}, use) => {
        appendFileSync('lifecycle.marker', 'setup;')
        await use({ timeout: 60000 })
        appendFileSync('lifecycle.marker', 'teardown;')
      }, { scope: '${scope}' }] })
      child.for([1, 2])('scoped %s', { timeout: 30000 }, (value, { sample, ownedRunnerMarker }) => {
        if (sample.timeout !== 60000 || ownedRunnerMarker !== 'preserved')
          throw new Error('scoped fixture/context changed')
        appendFileSync('lifecycle.marker', 'body;')
        continued()
      })
    `);
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
          source: `export { register } from 'deadline-registrar'`,
          transitiveSource: `import { beforeAll, test } from 'vitest';
          export function register(body) { beforeAll(() => {}, 30001); test('legal child', body, 25) }`,
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
      source: `export { register } from 'deadline-registrar'`,
      transitiveSource: `import { beforeAll, test } from 'vitest';
          export function register(body) { beforeAll(() => {}, 30001); test('legal child', body, 25) }`,
    });
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
});
describe("started SDK child failure finalization", () => {
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
