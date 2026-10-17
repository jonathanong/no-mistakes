export type Registration = (...args: never[]) => unknown;
export type Kind = "test" | "suite" | "hook";
/** Validate before Vitest's registration creates its captured timeout wrapper. */
export declare function guardRegistration<T extends Registration>(
  original: T,
  kind: Kind,
  maximum?: number,
): T;
