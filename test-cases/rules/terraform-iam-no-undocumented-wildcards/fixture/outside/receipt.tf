data "aws_iam_policy_document" "outside" {
  statement {
    resources = ["*"]
  }
}
