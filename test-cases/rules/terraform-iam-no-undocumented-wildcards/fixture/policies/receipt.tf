data "aws_iam_policy_document" "receipt_reader" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }

  statement {
    effect  = ("Allow")
    actions = ["s3:GetObject"]
    resources = ([
      ("arn:aws:s3:::example.invalid/receipts/202?-report.json"),
      "arn:aws:s3:::example.invalid/receipts/archive?.json",
      var.extra_receipt_arn,
    ])
  }

  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::${var.bucket}/*"]
  }
}
