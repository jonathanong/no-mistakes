type Key = string;
export type Mixed = Pick<typeof import("../src/mixed-union.mjs"), "mixedKey" | Key>;
