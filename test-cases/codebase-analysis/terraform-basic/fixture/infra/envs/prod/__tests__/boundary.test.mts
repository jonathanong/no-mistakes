// Mentions aws_route53_record.foobar (trailing) and legacy_aws_route53_record.foo
// (leading) — both merely embed the foo record's address inside a longer
// identifier and must NOT be treated as covering it.
import { test, expect } from "vitest";

test("legacy_aws_route53_record.foo is unrelated", () => {
  expect(true).toBe(true);
});
