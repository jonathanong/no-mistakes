# Root module. Consumes the network child module and declares two resources that
# reference each other, plus a local and a data source — exercising every
# reference kind the analyzer classifies.
module "network" {
  source = "../../modules/network"
  region = var.region
}

resource "aws_route53_record" "foo" {
  zone_id = module.network.zone_id
  name    = "foo.${data.aws_caller_identity.current.account_id}.example.com"
}

resource "aws_lb" "web" {
  name     = aws_route53_record.foo.name
  internal = local.is_internal
}

locals {
  is_internal = false
}

data "aws_caller_identity" "current" {}
