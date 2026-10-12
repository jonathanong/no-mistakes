import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe("public collector and data factory SDK contracts", () => {
  it.each(["beforeAll", "afterAll"])(
    "guards returned collector.test.%s before native registration",
    async (hook) => {
      const result = await fixture(`
      const collector = describe('owned', () => test('body', continued))
      collector.test.${hook}(async () => {}, 30001)
    `);
      expect(result.code, result.output).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it.each(["describe.each", "suite.for"])(
    "retains hook-shaped callable table rows through %s",
    async (registration) => {
      const result = await fixture(`
      const row = Object.assign(() => 'row', { beforeAll() {} })
      const metadata = new WeakMap([[row, 'owned metadata']])
      ${registration}([row])('row suite', (value) => {
        if (value !== row || metadata.get(value) !== 'owned metadata')
          throw new Error('CALLABLE_ROW_IDENTITY_CHANGED')
        test('body', continued)
      })
    `);
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it("preserves private static APIs on actual public SDK runner subclasses", async () => {
    const result = await fixture(
      fixtureSource("review-public-surfaces.test-18.txt"),
      undefined,
      `
      class StaticRunner extends TestRunner {
        static #marker = 'SDK static brand'
        static get marker() { return this.#marker }
        static read() { return this.#marker }
        static owner() { return this }
      }
      const Final = guardVitestExports({ TestRunner: StaticRunner }).TestRunner
      if (Final.marker !== 'SDK static brand' || Final.read() !== 'SDK static brand' ||
          Final.owner() !== StaticRunner) throw new Error('SDK_STATIC_RECEIVER_CHANGED')
      class Derived extends Final {
        static #marker = 'derived SDK static brand'
        static get marker() { return this.#marker }
        static read() { return this.#marker }
      }
      if (Derived.marker !== 'derived SDK static brand' ||
          Derived.read() !== 'derived SDK static brand' || Derived.owner() !== Derived)
        throw new Error('DERIVED_SDK_STATIC_RECEIVER_CHANGED')
    `,
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("retains valid returned-test hooks and rejects overcap test registration", async () => {
    const result = await fixture(fixtureSource("review-public-surfaces.test-19.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
});
