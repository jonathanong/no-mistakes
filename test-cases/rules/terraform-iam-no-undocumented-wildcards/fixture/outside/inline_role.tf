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

resource "aws_s3_bucket" "unrelated" {
  bucket = "example-bucket"
}
