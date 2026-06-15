# Infrastructure (Terraform/OpenTofu)

The `infra.terraform` block enables the [`infra`](../cli/infra.md) command. No
Terraform analysis runs unless `moduleRoots` is set — there are no default-on
conventions.

```yaml
infra:
  terraform:
    # Directories that are Terraform/OpenTofu modules (root or reusable child
    # modules). Each directory's `.tf` files are grouped as one module.
    moduleRoots:
      - infra/envs/prod
      - infra/modules/network
    # File extensions to treat as Terraform sources. Defaults to ["tf"].
    # `.tf.json` is not parsed (HCL native syntax only).
    extensions: [tf]
    # How `infra test-for` maps a `.tf` file to its covering tests.
    test:
      # Globs locating a module's tests, anchored at the module directory.
      testGlobs:
        - __tests__/*.test.mts
      # Optional: anchor `testGlobs` at this repo-root-relative directory instead.
      # testRoot: tests
      # "resource" (default) keeps only tests referencing an address declared in
      # the `.tf` file; "module" returns every test in the module directory.
      match: resource
```

## Limits

Parsing is structural HCL only (via `hcl-rs`); expressions are never evaluated.
`for_each`/`count` are not expanded and registry/remote module sources are not
followed. These are intentional heuristic limits.
