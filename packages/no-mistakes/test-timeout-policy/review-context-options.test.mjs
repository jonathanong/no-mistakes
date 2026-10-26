import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
import { browserFixture } from "../../../fixtures/test-timeout-policy/browser-timeout-child.mjs";
describe.skipIf(process.platform !== "linux")("public context, options and nested aliases", () => {
  it.each(["onTestFinished", "onTestFailed"])(
    "guards %s before a custom extension captures its timer",
    async (hook) => {
      const result = await fixture(
        fixtureSource("review-context-options.test-1.txt"),
        fixtureTemplate("review-context-options.test-template-5.txt"),
        fixtureTemplate("review-context-options.test-template-4.txt", [hook]),
        undefined,
        { policyMaximum: 20 },
      );
      expect(result.code, result.output).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite and at most 20 ms");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("rejects an enumerable inherited timeout that Vitest copies onto the task", async () => {
    const result = await fixture(
      fixtureTemplate("review-context-options.test-template-3.txt", [
        fixtureTemplate("review-context-options.test-extra-template-4.txt"),
        fixtureTemplate("review-context-options.test-extra-template-5.txt"),
      ]),
    );
    expect(result.code, result.output).toBe(1);
    expect(result.output).toContain(
      "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
    );
    expect(result.forcedCleanup).toBe(false);
  });
  it("reports a getter Vitest reads from an inherited timeout", async () => {
    const result = await fixture(
      fixtureTemplate("review-context-options.test-template-3.txt", [
        fixtureTemplate("review-context-options.test-extra-template-5.txt"),
        fixtureTemplate("review-context-options.test-extra-template-2.txt"),
      ]),
    );
    expect(result.code, result.output).toBe(1);
    expect(result.output).toContain("IGNORED_GETTER_READ");
    expect(result.forcedCleanup).toBe(false);
  });
  it("preserves native registrations when the SDK ignores non-enumerable timeout properties", async () => {
    const result = await fixture(
      fixtureTemplate("review-context-options.test-template-3.txt", [
        fixtureTemplate("review-context-options.test-extra-template-3.txt"),
        fixtureTemplate("review-context-options.test-extra-template-1.txt"),
      ]),
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("preserves valid hooks on a returned replacement context", async () => {
    const result = await fixture(
      fixtureSource("review-context-options.test-2.txt"),
      fixtureTemplate("review-context-options.test-template-2.txt"),
      fixtureTemplate("review-context-options.test-template-1.txt"),
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
  describe.skipIf(process.platform !== "linux")(
    "Browser Mode reporter environment",
    { timeout: 30000 },
    () => {
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
    },
  );
});
