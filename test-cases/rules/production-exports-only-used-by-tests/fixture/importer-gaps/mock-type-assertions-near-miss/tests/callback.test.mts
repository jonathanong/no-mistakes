// An unrelated callback is not a mock factory.
export const items = [1].map(() => ({}) as typeof import("../src/callback.mjs"));
