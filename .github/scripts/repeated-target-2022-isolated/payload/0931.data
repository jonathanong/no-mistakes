export { guardRegistration, type Registration, type Kind } from "./registration.mjs";
export { guardPublicRunner, type RunnerConstructor, type Protector } from "./public-runner.mjs";
export {
  assertTestTimeout,
  assertTimeoutConfig,
  snapshotTimeoutOptions,
  protectTimeoutConfig,
} from "./config.mjs";
/** Aliased/destructured setters retain validation because the returned method is guarded. */
export declare function guardRuntimeConfig<T extends object>(api: T, maximum?: number): T;
/** A single current Vitest API facade. Adoption must route every test import through it. */
export declare function guardVitestExports<T extends object>(api: T, maximum?: number): T;
/** Compose at runner construction, without replacing existing collection/database guards. */
export declare function protectRunnerTimeouts(
  runner: {
    config: object;
  },
  maximum?: number,
): void;
