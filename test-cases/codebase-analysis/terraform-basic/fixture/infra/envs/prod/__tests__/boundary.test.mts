// Mentions aws_route53_record.foobar, a different resource whose name merely
// extends the foo record's name — it must NOT be treated as covering the
// shorter address.
import { test, expect } from "vitest";

test("aws_route53_record.foobar is unrelated", () => {
  expect(true).toBe(true);
});
