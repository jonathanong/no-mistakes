import { expect, test } from "vitest";
const text = error.message;
// no-mistakes-disable-next-line test-no-error-message-matching -- owns copy
expect(text).toBe("missing");
expect(text).toBe("missing"); // no-mistakes-disable-line no-mistakes/test-no-error-message-matching -- owns copy
test("copy", () => {
  // no-mistakes-disable-next-line test-no-error-message-matching -- owns copy
  text.includes("missing");
  text === "missing"; // no-mistakes-disable-line test-no-error-message-matching -- owns copy
});
