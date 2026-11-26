data "aws_iam_policy_document" "bucket_catalog" {
  statement {
    actions = ["s3:ListAllMyBuckets"]
    # no-mistakes-disable-next-line terraform-iam-no-undocumented-wildcards: This action requires all resources.
    resources = ["*"]
  }

  statement {
    actions   = ["s3:ListAllMyBuckets"]
    resources = ["*"] # no-mistakes-disable-line terraform-iam-no-undocumented-wildcards: This action requires all resources.
  }
}
