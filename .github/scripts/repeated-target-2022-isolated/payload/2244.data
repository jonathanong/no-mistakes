import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
const inlineAll = fixtureTemplate("review-regressions.test-template-6.txt");
describe.skipIf(process.platform !== "linux")("public SDK review regressions", () => {
  it("preserves inline-all dependency configuration through actual SDK collection", async () => {
    const result = await fixture(fixtureSource("review-regressions.test-38.txt"), inlineAll);
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("still guards an inlined package registrar with the inline-all sentinel", async () => {
    const result = await fixture(fixtureSource("review-regressions.test-39.txt"), inlineAll, "", {
      preserveSymlinks: false,
      source: fixtureTemplate("review-regressions.test-extra-template-1.txt"),
    });
    expect(result.code).toBe(1);
    expect(result.output).toContain("hook timeout must be positive, finite");
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it("preserves direct new.target in the actual worker-loaded final constructor", async () => {
    const result = await fixture(
      fixtureSource("review-regressions.test-40.txt"),
      undefined,
      "",
      undefined,
      {},
      { assertDirectNewTarget: true },
    );
    expect(result.code, result.output).toBe(0);
    expect(result.lifecycle).toBe("final constructor initialized");
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["beforeAll", "afterAll"])(
    fixtureTemplate("review-regressions.test-template-5.txt"),
    async (hook) => {
      const result = await fixture(
        fixtureTemplate("review-regressions.test-template-4.txt", [hook]),
        fixtureTemplate("review-regressions.test-template-3.txt"),
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("SuiteCollector.on bypasses native hook deadlines");
      expect(result.output).not.toContain("RAW_HOOK_EXECUTED");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("preserves returned collector test, task and clear APIs during native collection", async () => {
    const result = await fixture(fixtureSource("review-regressions.test-41.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.output).not.toContain("CLEARED_TEST_EXECUTED");
    expect(result.forcedCleanup).toBe(false);
  });
  it("uses the configured native deadline for guarded public hooks inside the suite factory", async () => {
    const result = await fixture(
      fixtureSource("review-regressions.test-42.txt"),
      fixtureTemplate("review-regressions.test-template-2.txt"),
    );
    expect(result.code).toBe(1);
    expect(result.output).toContain("Hook timed out in 20ms");
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it("preserves guarded suite-factory hooks with the configured native deadline", async () => {
    const result = await fixture(
      fixtureSource("review-regressions.test-43.txt"),
      fixtureTemplate("review-regressions.test-template-1.txt"),
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
});
