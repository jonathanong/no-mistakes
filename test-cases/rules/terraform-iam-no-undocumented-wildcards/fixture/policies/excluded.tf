data "aws_iam_policy_document" "excluded" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }
}
