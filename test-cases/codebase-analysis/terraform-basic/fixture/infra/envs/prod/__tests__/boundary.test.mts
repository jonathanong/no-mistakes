// Mentions aws_route53_record.foobar (trailing), legacy_aws_route53_record.foo
// (leading), and aws_route53_record.foo-logs (dashed) — all merely embed the foo
// record's address inside a longer identifier and must NOT cover it.
import { test, expect } from "vitest";

test("aws_route53_record.foo-logs is unrelated", () => {
  expect(true).toBe(true);
});
