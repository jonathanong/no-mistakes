# Intentionally invalid HCL (unterminated block) to exercise parse-failure
# reporting. Do not "fix" this — the malformed syntax is the test invariant.
resource "aws_s3_bucket" "broken" {
  bucket =
