data "aws_iam_policy_document" "one_line" {
  statement { resources = ["arn:aws:s3:::example.invalid/&{aws:username}/*"] }
}
