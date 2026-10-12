import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe("candidate public TestRunner constructor composition (Node probes)", () => {
  it("preserves cached identity, most-derived prototype and custom context lifecycle", async () => {
    const result = await fixture(
      `test('body', continued)`,
      undefined,
      `
      const Guarded = guardVitestExports({ TestRunner }).TestRunner
      if (Guarded !== guardVitestExports({ TestRunner }).TestRunner)
        throw new Error('constructor cache identity lost')
      if (Guarded.prototype !== TestRunner.prototype) throw new Error('public prototype changed')
      for (const key of Reflect.ownKeys(TestRunner)) {
        if (Reflect.get(Guarded, key) !== Reflect.get(TestRunner, key))
          throw new Error('public static property changed')
      }
      class Derived extends Guarded {
        static marker = 'static preserved'
        constructor(config) { super(config); this.marker = 'instance preserved' }
        importFile(path) { return path }
        extendTaskContext(context) {
          const extended = super.extendTaskContext(context)
          extended.derivedMarker = this.marker
          return extended
        }
      }
      if (Object.getPrototypeOf(Derived) !== Guarded)
        throw new Error('static constructor inheritance changed')
      const runner = new Derived({ ...this.config })
      runner.onBeforeCollect([])
      for (const name of ['onCollectStart', 'onAfterRunSuite', 'onBeforeRunTask',
          'onBeforeTryTask', 'onAfterTryTask', 'cancel', 'injectValue',
          'getImportDurations', 'getModuleFetchDuration']) {
        if (Reflect.get(runner, name) !== Reflect.get(TestRunner.prototype, name))
          throw new Error('public SDK lifecycle method replaced: ' + name)
      }
      if (!(runner instanceof Derived) || !(runner instanceof TestRunner) ||
          Object.getPrototypeOf(runner) !== Derived.prototype ||
          Derived.marker !== 'static preserved' || runner.importFile('owned') !== 'owned')
        throw new Error('most-derived public API changed')
      const Lower = guardVitestExports({ TestRunner }, 100).TestRunner
      if (Lower === Guarded) throw new Error('configured maximum cache isolation lost')
      const lower = new Lower({ ...this.config, testTimeout: 25, hookTimeout: 25 })
      lower.config.testTimeout = 100
      try { lower.config.testTimeout = 101; throw new Error('lower ceiling lost') }
      catch (error) { if (!(error instanceof RangeError)) throw error }
      const extended = runner.extendTaskContext({ task: { timeout: 25 } })
      if (extended.derivedMarker !== 'instance preserved') throw new Error('context lifecycle lost')
      try { runner.config.testTimeout = 30001; throw new Error('invalid write accepted') }
      catch (error) { if (!(error instanceof RangeError)) throw error }
    `,
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("guards the final public SDK subclass with an own-field extension and private receiver", async () => {
    const result = await fixture(
      `test('body', continued)`,
      undefined,
      `
      class Derived extends TestRunner {
        #marker = 'derived ready'
        config = { ...this.config, testTimeout: 25, hookTimeout: 25 }
        extendTaskContext = (context) => {
          const extended = super.extendTaskContext(context)
          extended.marker = this.#marker
          return extended
        }
      }
      const Final = guardVitestExports({ TestRunner: Derived }).TestRunner
      const runner = new Final({ ...this.config })
      const finalizedContext = runner.extendTaskContext({ task: { timeout: 25 } })
      if (finalizedContext.marker !== 'derived ready' || !(runner instanceof Derived))
        throw new Error('own-field public SDK receiver lost')
      try { runner.extendTaskContext({ task: { timeout: 30001 } }); throw new Error('task accepted') }
      catch (error) { if (!(error instanceof RangeError)) throw error }
      try { runner.config.hookTimeout = 30001; throw new Error('hook accepted') }
      catch (error) { if (!(error instanceof RangeError)) throw error }
    `,
    );
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each(["30001", "0", "NaN", "Infinity"])(
    "rejects raw configured deadline %s during public construction",
    async (timeout) => {
      const result = await fixture(
        `test('body', continued)`,
        undefined,
        `
        const Guarded = guardVitestExports({ TestRunner }).TestRunner
        new Guarded({ ...this.config, testTimeout: ${timeout} })
      `,
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("testTimeout must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    },
  );
});
