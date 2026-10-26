// no-mistakes-disable-file test-no-error-message-matching -- owns copy
import { expect, test } from "vitest";
const text = error.message;
expect(text).toBe("missing");
test("copy", () => {
  text.includes("missing");
  text === "missing";
});
