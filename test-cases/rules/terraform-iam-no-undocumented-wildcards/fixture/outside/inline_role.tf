data "aws_iam_policy_document" "edge_cases" {
  statement "labeled" {
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }

  statement {
    actions = ["s3:GetObject"]
    resources = [
      "arn:aws:s3:::example.invalid/&{unclosed",
      <<-EOT
      arn:aws:s3:::example.invalid/receipts/clean.json
    EOT
    ]
  }

  statement {
    actions   = ["s3:GetObject"]
    resources = [var.single_arn]
  }
}

resource "aws_iam_role" "worker" {
  name = "worker"

  inline_policy {
    name = "inline-json"
    policy = jsonencode({
      Version = "2012-10-17"
      Statement = [{
        Effect   = "Allow"
        Action   = ["s3:GetObject"]
        Resource = "*"
      }]
    })
  }

  dynamic "inline_policy" {
    for_each = var.extra_policies
    content {
      name   = inline_policy.key
      policy = inline_policy.value
    }
  }
}

resource "aws_iam_role_policy" "var_policy" {
  name   = "var-policy"
  role   = "worker"
  policy = var.policy_json
}

resource "aws_iam_role_policy" "indexed_doc" {
  name   = "indexed-doc"
  role   = "worker"
  policy = data.aws_iam_policy_document.edge_cases[0].json
}

resource "aws_s3_bucket" "unrelated" {
  bucket = "example-bucket"
}
