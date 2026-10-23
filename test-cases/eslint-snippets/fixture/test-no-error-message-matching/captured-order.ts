import { expect } from "vitest";
declare const error: Error;
// These lexical reads do not prove an initialized value, even in dead code.
expect(early).toBe("TDZ");
const early = error.message;
const fromLater = later;
const later = error.message;
expect(fromLater).toBe("TDZ");
const cycleA = cycleB;
const cycleB = cycleA;
expect(cycleA).toBe("cycle");
const changed = error.message;
changed = "replacement";
expect(changed).toBe("invalid write");
const first = error.message;
first = "replacement";
const second = first;
expect(second).toBe("invalid origin write");
const stillMissing = unknown;
expect(stillMissing).toBe("unresolved");
checkBeforeInitialization();
const hoistedText = error.message;
function checkBeforeInitialization() {
  expect(hoistedText).toBe("TDZ through hoisting");
}
const numberCast = error.message as unknown as number;
expect(numberCast).toBe(42);
