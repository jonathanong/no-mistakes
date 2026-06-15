variable "region" {}

variable "zone_name" {
  default = "example.com"
}

resource "aws_route53_zone" "main" {
  name = var.zone_name
}
