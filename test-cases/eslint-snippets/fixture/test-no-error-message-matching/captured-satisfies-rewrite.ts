import { expect, test } from "vitest";
declare const error: Error & { code: string };
// Preserve the original capture point and erased type check.
const code = error.code satisfies string;
expect(code).toBe("MISSING");
const alias = code satisfies unknown;
expect(alias satisfies string).toBe("MISSING");
(expect satisfies typeof expect)(error.code).toBe("MISSING");
test("captured satisfies", () => {
  alias === "MISSING";
  (alias satisfies string) === "MISSING";
});
