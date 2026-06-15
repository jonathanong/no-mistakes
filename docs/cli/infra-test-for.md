# `no-mistakes infra test-for`

List the test files that cover the resources defined in a Terraform/OpenTofu file.

```sh
no-mistakes infra test-for infra/envs/prod/main.tf --format paths
```

The test convention is configuration-driven — nothing is hardcoded. Set
`infra.terraform.test.testGlobs` (anchored at the module directory, or at
`testRoot` when set). With `test.match: resource` (the default) only tests whose
contents reference an address declared in the `.tf` file are returned; with
`test.match: module` every test in the module directory is returned.

Node API: `infraTestFor(options)`.
