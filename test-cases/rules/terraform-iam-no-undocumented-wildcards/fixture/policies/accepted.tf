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

resource "aws_iam_policy" "local_doc" {
  name   = "local-doc"
  policy = data.aws_iam_policy_document.scoped.json
}

resource "aws_iam_policy" "json_inline" {
  name = "json-inline"
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["s3:GetObject"]
      Resource = "*"
    }]
  })
}

resource "aws_iam_role_policy" "external_doc" {
  name   = "external-doc"
  role   = "example"
  policy = data.aws_iam_policy_document.shared.json
}

data "aws_iam_policy_document" "merged" {
  source_policy_documents = [var.base_policy]
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::example.invalid/receipts/current.json"]
  }
}
