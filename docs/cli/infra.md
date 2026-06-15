# `no-mistakes infra`

Analyze Terraform/OpenTofu resource, module, and output relationships from the
`.tf` file graph. Parsing is structural (HCL syntax via `hcl-rs`, never evaluated),
so `for_each`/`count` are not expanded and registry/remote modules are not
followed — a documented heuristic limit. `.tf.json` is not parsed.

Configure which directories are modules via `infra.terraform.moduleRoots`; no
analysis happens without it.

```yaml
infra:
  terraform:
    moduleRoots:
      - infra/envs/prod
      - infra/modules/network
    test:
      testGlobs:
        - __tests__/*.test.mts
      match: resource
```

## Subcommands

| Command | Purpose |
| --- | --- |
| [`infra resource-refs`](infra-resource-refs.md) | Resources/modules/outputs that reference a `<type>.<name>`. |
| [`infra outputs`](infra-outputs.md) | Outputs a module exports and the root modules that consume them. |
| [`infra test-for`](infra-test-for.md) | Test files covering resources defined in a `.tf` file. |

See [`Graph edges`](../graph-edges.md) for the `terraform-ref`, `terraform-module`,
and `terraform-output` edge kinds these queries are built on.
