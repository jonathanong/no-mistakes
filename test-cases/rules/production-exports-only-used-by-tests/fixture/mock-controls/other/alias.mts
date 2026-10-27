import { vi as mockApi } from "vitest";
// Vitest default transformation recognizes vi/vitest spellings, not arbitrary aliases.
mockApi.mock(import("../src/alias.mjs"), () => ({ aliasValue: 2 }));
