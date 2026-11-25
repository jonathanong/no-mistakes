data "aws_iam_policy_document" "scoped" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::example.invalid/receipts/current&total$.json"]
  }
  statement {
    effect    = "Deny"
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }
  statement {
    effect    = var.effect
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }
  statement {
    actions   = ["s3:GetObject"]
    resources = var.resources
  }
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::example.invalid/receipts/${var.file}"]
  }
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::example.invalid/receipts/&{*}&{?}$${*}$${?}.json"]
  }
  statement {
    actions       = ["s3:GetObject"]
    not_resources = ["*"]
  }
  dynamic "statement" {
    for_each = var.policies
    content {
      actions   = ["s3:GetObject"]
      resources = ["*"]
    }
  }
}

data "example_document" "unrelated" {
  statement {
    resources = ["*"]
  }
}

locals {
  example = <<-EOT
  data "aws_iam_policy_document" "inside_string" {
    statement { resources = ["*"] }
  }
  EOT
}
