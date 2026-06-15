# `no-mistakes infra outputs`

Show the outputs a Terraform/OpenTofu module exports and the root/parent modules
that consume them via `module.<name>.<output>` references.

```sh
no-mistakes infra outputs infra/modules/network --format json
```

`exports` lists each output and the addresses its `value` references. `consumers`
lists every `module.<name>.<output>` reference whose module `source` resolves to
this directory. Requires `infra.terraform.moduleRoots` in config.

Node API: `infraOutputs(options)`.
