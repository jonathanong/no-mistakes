import { expect, test } from "vitest";
declare const error: Error;
// `satisfies` is erased and keeps the captured string's identity.
const text = error.message satisfies string;
expect(text).toBe("missing");
const alias = text satisfies unknown;
expect(alias satisfies string).toContain("missing");
(expect satisfies typeof expect)(error.message).toBe("missing");
test("captured satisfies", () => {
  alias.includes("missing");
  (alias satisfies string) === "missing";
});
