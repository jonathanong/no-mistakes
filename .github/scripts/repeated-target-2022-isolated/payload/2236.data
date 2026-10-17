import { describe, expect, it, vi } from "vitest";
import { guardPublicRunner } from "./public-runner.mjs";
import { protectRunnerTimeouts } from "./index.mjs";
describe("public constructor adapter contracts (parent worker)", () => {
  it("preserves private-field receivers and prototype delegation after derived initialization", () => {
    const calls = vi.fn();
    class Base {
      config;
      #identity = "base";
      constructor(config) {
        this.config = config;
      }
      baseIdentity() {
        return this.#identity;
      }
      extendTaskContext(context) {
        return context;
      }
      static label = "public static";
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    class Derived extends Guarded {
      #identity = "derived";
      constructor(config) {
        super(config);
        // SDK5 browser lifecycle: a temporary field reset, then the original object.
        Object.defineProperty(this, "config", {
          value: undefined,
          configurable: true,
          writable: true,
          enumerable: true,
        });
        this.config = config;
      }
      extendTaskContext(context) {
        calls(this.#identity, this.baseIdentity());
        return super.extendTaskContext(context);
      }
    }
    const config = { testTimeout: 25, hookTimeout: 25 };
    const runner = new (guardPublicRunner(Derived, 30000, protectRunnerTimeouts))(config);
    expect(runner).toBeInstanceOf(Derived);
    expect(runner).toBeInstanceOf(Base);
    expect(runner.config).toBe(config);
    expect(Derived.label).toBe("public static");
    expect(Object.getPrototypeOf(Derived)).toBe(Guarded);
    expect(Object.getPrototypeOf(runner)).toBe(Derived.prototype);
    expect(() => runner.extendTaskContext({ task: { timeout: 0 } })).toThrow(RangeError);
    expect(calls).not.toHaveBeenCalled();
    const context = { task: { timeout: 25 } };
    expect(runner.extendTaskContext(context)).toBe(context);
    expect(calls).toHaveBeenCalledWith("derived", "base");
    expect(() => {
      config.hookTimeout = 30001;
    }).toThrow(RangeError);
  });
  it("preserves direct and genuinely derived constructor new.target values", () => {
    class Base {
      config;
      target;
      #marker = "base brand";
      constructor(config) {
        this.config = config;
        this.target = new.target;
      }
      marker() {
        return this.#marker;
      }
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    const direct = new Guarded({ testTimeout: 25, hookTimeout: 25 });
    expect(direct.target).toBe(Base);
    expect(direct.marker()).toBe("base brand");
    class Derived extends Guarded {
      #derived = "derived brand";
      derived() {
        return this.#derived;
      }
    }
    const derived = new Derived({ testTimeout: 25, hookTimeout: 25 });
    expect(derived.target).toBe(Derived);
    expect(derived.marker()).toBe("base brand");
    expect(derived.derived()).toBe("derived brand");
    expect(derived).toBeInstanceOf(Derived);
  });
  it("retains direct static private brands, getters, method identity and derived receivers", () => {
    class Base {
      config;
      static #marker = "base static brand";
      static get marker() {
        return this.#marker;
      }
      static read() {
        return this.#marker;
      }
      static owner() {
        return this;
      }
      static callableField = () => "public callable field";
      static get ownerMarker() {
        return this;
      }
      constructor(config) {
        this.config = config;
      }
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    expect(Reflect.get(Guarded, "callableField")).toBe(Reflect.get(Base, "callableField"));
    expect(Guarded.callableField()).toBe("public callable field");
    expect(Guarded.marker).toBe(Base.marker);
    expect(Guarded.read()).toBe(Base.read());
    expect(Reflect.get(Guarded, "read")).toBe(Reflect.get(Guarded, "read"));
    const read = Reflect.get(Guarded, "read");
    if (typeof read !== "function") throw new Error("Static method missing");
    expect(() => read()).toThrow(TypeError);
    expect(() => read.call({})).toThrow(TypeError);
    class Derived extends Guarded {
      static #marker = "derived static brand";
      static get marker() {
        return this.#marker;
      }
      static read() {
        return this.#marker;
      }
    }
    expect(Derived.marker).toBe("derived static brand");
    expect(Derived.read()).toBe("derived static brand");
    expect(Derived.owner()).toBe(Derived);
    expect(Derived.ownerMarker).toBe(Derived);
    expect(Guarded.ownerMarker).toBe(Base);
    expect(Reflect.get(Derived, "callableField")).toBe(Reflect.get(Base, "callableField"));
    class Inherited extends Base {}
    const FinalInherited = guardPublicRunner(Inherited, 30000, protectRunnerTimeouts);
    expect(Reflect.get(FinalInherited, "callableField")).toBe(Reflect.get(Base, "callableField"));
    expect(FinalInherited.owner()).toBe(Inherited);
    const Bound = Guarded.bind(null);
    expect(() => new Bound({ testTimeout: 30001, hookTimeout: 25 })).toThrow(RangeError);
    expect(Guarded.owner()).toBe(Base);
    const Frozen = class {
      config;
      constructor(config) {
        this.config = config;
      }
      static fixed() {
        return "immutable";
      }
    };
    Object.defineProperty(Frozen, "fixed", { writable: false, configurable: false });
    const guardedFrozen = guardPublicRunner(Frozen, 30000, protectRunnerTimeouts);
    expect(guardedFrozen.fixed).toBe(Frozen.fixed);
    expect(guardedFrozen.fixed()).toBe("immutable");
    class FrozenDerived extends guardedFrozen {}
    expect(Reflect.get(FrozenDerived, "fixed")).toBe(Reflect.get(Frozen, "fixed"));
    expect(FrozenDerived.fixed()).toBe("immutable");
  });
  it("shares adapters per constructor and protector but isolates configured maxima", () => {
    class Base {
      config;
      constructor(config) {
        this.config = config;
      }
    }
    const protect = vi.fn(protectRunnerTimeouts);
    const first = guardPublicRunner(Base, 30000, protect);
    expect(guardPublicRunner(Base, 30000, protect)).toBe(first);
    const lower = guardPublicRunner(Base, 100, protect);
    expect(lower).not.toBe(first);
    expect(guardPublicRunner(Base, 100, protect)).toBe(lower);
    const other = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    expect(other).not.toBe(first);
    const config = { testTimeout: 25, hookTimeout: 25 };
    const instance = new lower(config);
    expect(instance.config).toBe(config);
    config.testTimeout = 100;
    expect(() => {
      config.testTimeout = 101;
    }).toThrow(RangeError);
    expect(config.testTimeout).toBe(100);
    expect(protect).toHaveBeenCalledWith(instance, 100);
  });
  it("rejects changed config identity before delegated context/body work", () => {
    const calls = vi.fn((context) => context);
    class Base {
      config;
      constructor(config) {
        this.config = config;
      }
      extendTaskContext(context) {
        return calls(context);
      }
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    class Derived extends Guarded {
      config = { testTimeout: 25, hookTimeout: 25 };
    }
    const runner = new (guardPublicRunner(Derived, 30000, protectRunnerTimeouts))({
      testTimeout: 25,
      hookTimeout: 25,
    });
    runner.config = { testTimeout: 25, hookTimeout: 25 };
    expect(() => runner.extendTaskContext({ task: { timeout: 25 } })).toThrow(
      "configuration identity changed",
    );
    expect(calls).not.toHaveBeenCalled();
    // This does not prove collection hooks captured before this call were protected.
  });
  it("rejects a delegated config replacement before exposing the returned context", () => {
    const replacement = { task: { timeout: 25 } };
    class Base {
      config;
      constructor(config) {
        this.config = config;
      }
      extendTaskContext(context) {
        return context;
      }
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    class Derived extends Guarded {
      extendTaskContext(context) {
        expect(context).not.toBe(replacement);
        this.config = { testTimeout: 25, hookTimeout: 25 };
        return replacement;
      }
    }
    const runner = new (guardPublicRunner(Derived, 30000, protectRunnerTimeouts))({
      testTimeout: 25,
      hookTimeout: 25,
    });
    expect(() => runner.extendTaskContext({ task: { timeout: 25 } })).toThrow(
      "identity changed during context extension",
    );
  });
  it("validates the original task after delegation even when a different context is returned", () => {
    const replacement = { task: { timeout: 25 } };
    class Base {
      config;
      constructor(config) {
        this.config = config;
      }
      extendTaskContext(context) {
        return context;
      }
    }
    const Guarded = guardPublicRunner(Base, 30000, protectRunnerTimeouts);
    class Derived extends Guarded {
      extendTaskContext(context) {
        context.task.timeout = 30001;
        return replacement;
      }
    }
    const runner = new (guardPublicRunner(Derived, 30000, protectRunnerTimeouts))({
      testTimeout: 25,
      hookTimeout: 25,
    });
    expect(() => runner.extendTaskContext({ task: { timeout: 25 } })).toThrow(RangeError);
  });
  it("finalizes an actual custom constructor after own fields without changing private receivers", () => {
    class Base {
      config;
      constructor(config) {
        this.config = config;
      }
      extendTaskContext(context) {
        return context;
      }
    }
    class Derived extends Base {
      #marker = "initialized";
      config = { testTimeout: 25, hookTimeout: 25 };
      extendTaskContext = (context) => {
        context.marker = this.#marker;
        return context;
      };
    }
    // Compose the actual final constructor, not its base: all own fields exist here.
    const Final = guardPublicRunner(Derived, 30000, protectRunnerTimeouts);
    const runner = new Final({ testTimeout: 25, hookTimeout: 25 });
    const context = { task: { timeout: 25 } };
    expect(runner.extendTaskContext(context)).toBe(context);
    expect(context.marker).toBe("initialized");
    expect(runner).toBeInstanceOf(Derived);
    expect(() => runner.extendTaskContext({ task: { timeout: 30001 } })).toThrow(RangeError);
    expect(() => {
      runner.config.hookTimeout = 30001;
    }).toThrow(RangeError);
  });
});
