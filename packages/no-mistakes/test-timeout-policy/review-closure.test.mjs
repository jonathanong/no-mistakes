import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe.skipIf(process.platform !== "linux")("public SDK policy closure regressions", () => {
  it.each(["testTimeout", "hookTimeout"])("guards default two-argument Node %s", async (field) => {
    const result = await fixture(
      fixtureSource("review-closure.test-20.txt"),
      fixtureTemplate("review-closure.test-extra-template-4.txt", [field]),
      "",
      undefined,
      {},
      { nativeDefault: true, implicitNative: true },
    );
    expect(result.code, result.output).toBe(1);
    expect(result.output).toContain(
      fixtureTemplate("review-closure.test-extra-template-3.txt", [field]),
    );
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it("retains native aliases, runtime setter identity and valid implicit Node collection", async () => {
    const result = await fixture(
      fixtureSource("review-closure.test-21.txt"),
      undefined,
      "",
      undefined,
      {},
      { nativeDefault: true, implicitNative: true },
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["beforeAll", "afterAll", "beforeEach", "afterEach", "aroundAll", "aroundEach"])(
    "guards suite-factory injected %s registration before collection",
    async (hook) => {
      const result = await fixture(
        fixtureTemplate("review-closure.test-template-1.txt", [
          hook,
          hook.startsWith("around")
            ? fixtureTemplate("review-closure.test-extra-template-2.txt")
            : fixtureTemplate("review-closure.test-extra-template-1.txt"),
        ]),
      );
      expect(result.code, result.output).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("preserves asynchronous factory receiver, valid injected hooks and data-driven row identities", async () => {
    const result = await fixture(fixtureSource("review-closure.test-22.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
});
