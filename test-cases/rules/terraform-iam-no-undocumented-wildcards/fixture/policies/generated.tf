# no-mistakes-disable-file terraform-iam-no-undocumented-wildcards: Generated from a separately reviewed policy.
data "aws_iam_policy_document" "generated" {
  statement {
    actions   = ["s3:ListAllMyBuckets"]
    resources = ["*"]
  }
}
