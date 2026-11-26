import { it } from "vitest";

it("uses recipients", () => [
  "selected@example.com", // no-mistakes-disable-line test-email-domain-policy
  // no-mistakes-disable-next-line test-email-domain-policy
  "next@example.com",
  "sibling@example.com",
]);
