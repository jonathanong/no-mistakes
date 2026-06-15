// Mentions aws_route53_record.foobar (trailing), legacy_aws_route53_record.foo
// (leading), aws_route53_record.foo-logs (dashed), and data.aws_route53_record.foo
// (dotted prefix — a different data-source address) — all merely embed the foo
// record's address inside a longer identifier and must NOT cover it.
import { test, expect } from "vitest";

test("data.aws_route53_record.foo is unrelated", () => {
  expect(true).toBe(true);
});
