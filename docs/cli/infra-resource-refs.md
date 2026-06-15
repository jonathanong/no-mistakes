# `no-mistakes infra resource-refs`

List the resources, modules, and outputs that reference a given Terraform/OpenTofu
resource or data source.

```sh
no-mistakes infra resource-refs aws_route53_record.foo --format json
```

The address is the canonical `<type>.<name>` form (`aws_route53_record.foo`) or
`data.<type>.<name>` for data sources. Each result is a referencing block address
plus the file it lives in. Requires `infra.terraform.moduleRoots` in config.

Node API: `infraResourceRefs(options)`.
