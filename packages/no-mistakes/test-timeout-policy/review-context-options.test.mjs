import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
import { browserFixture } from "../../../fixtures/test-timeout-policy/browser-timeout-child.mjs";
describe("public context, options and nested aliases", () => {
  it.each(["onTestFinished", "onTestFailed"])(
    "guards %s before a custom extension captures its timer",
    async (hook) => {
      const result = await fixture(
        fixtureSource("review-context-options.test-1.txt"),
        "{ testTimeout: 20, hookTimeout: 20 }",
        `result.${hook}(() => {}, 150)`,
        undefined,
        { policyMaximum: 20 },
      );
      expect(result.code, result.output).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite and at most 20 ms");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it.each(["inherited", "non-enumerable"])(
    "preserves native registrations when the SDK ignores %s timeout properties",
    async (kind) => {
      const result = await fixture(`
        const options = ${kind === "inherited" ? "Object.create({ timeout: 30001 })" : "Object.defineProperty({}, 'timeout', { value: 30001 })"}
        native.test('native control', options, () => {})
        test('guarded default', options, () => {})
        describe('guarded suite default', options, () => test('body', continued))
        const ignored = ${kind === "inherited" ? "Object.create({ get timeout() { throw new Error('IGNORED_GETTER_READ') } })" : "Object.defineProperty({}, 'timeout', { get() { throw new Error('IGNORED_GETTER_READ') } })"}
        native.test('native ignored getter', ignored, () => {})
        test('guarded ignored getter', ignored, () => {})
      `);
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("preserves valid hooks on a returned replacement context", async () => {
    const result = await fixture(
      fixtureSource("review-context-options.test-2.txt"),
      "{ testTimeout: 20, hookTimeout: 20 }",
      `return Object.assign(function(...args) { return Reflect.apply(result, this, args) },
         result, { replacement: true })`,
      undefined,
      { policyMaximum: 20 },
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.extensionMarker).toBe("finished");
    expect(result.forcedCleanup).toBe(false);
  });
  it("retains native nested TestAPI aliases and their guarded callbacks", async () => {
    const result = await fixture(fixtureSource("review-context-options.test-3.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("retains a completion-method alias after native multi-hook execution", async () => {
    const result = await fixture(fixtureSource("review-context-options.test-4.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  describe("Browser Mode reporter environment", { timeout: 30000 }, () => {
    it.each([{ GITHUB_ACTIONS: "true" }, { AI_AGENT: "qualification" }])(
      "retains environment-native reporters through actual Browser Mode: %s",
      async (environment) => {
        const result = await browserFixture(
          fixtureSource("review-context-options.test-5.txt"),
          5000,
          undefined,
          undefined,
          environment,
        );
        expect(result.code, result.output).toBe(0);
        const receipt = JSON.parse(result.reporters);
        expect(receipt.resolved).toEqual([...receipt.defaults, "lifecycle"]);
        if ("GITHUB_ACTIONS" in environment) expect(receipt.defaults).toContain("github-actions");
        else expect(receipt.defaults).toContain("minimal");
        expect(result.forcedCleanup).toBe(false);
      },
    );
  });
});
