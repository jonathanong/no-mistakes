# `terraform-iam-no-undocumented-wildcards`

## Why and when

A wildcard in an IAM Allow statement can grant access to more resources than
the caller needs. Some AWS actions require a wildcard. Record why the action
needs it where reviewers can see the scope of the grant.
[AWS IAM resource reference](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_resource.html)

## Options and defaults

The rule is off until configured. It supports the shared `options.unanalyzable`
policy (`report` by default, `advise`, or `ignore`). Use standard `projects` or
`scope`, and `include` and `exclude`, to select Terraform or OpenTofu files
that declare AWS IAM policies.

```yaml
rules:
  - rule: terraform-iam-no-undocumented-wildcards
    scope: repository
    include: ["infra/policies/**/*.tf"]
    options:
      unanalyzable: report
```

## What it catches

The rule parses selected `.tf` files and inspects `statement` blocks in
`data "aws_iam_policy_document"` declarations. It reports a `resources`
attribute as a definite finding when the effect is explicitly `Allow` or
defaults to `Allow` and any literal array entry—or any literal segment of an
interpolated string such as `"arn:aws:s3:::${var.bucket}/*"`—contains a
wildcard (`*` or `?`). It reports once per attribute, even when several
entries contain wildcards. AWS policy variable expressions such as `&{*}` or
`$${*}` denote literal characters and do not count as wildcards.
[AWS IAM policy variables](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_variables.html#policy-vars-specialchars)

Explicit `Deny` statements and `not_resources` attributes do not trigger
findings. Any IAM policy expression or shape the rule cannot statically prove
clean is routed through the `unanalyzable` policy (`target: "unanalyzable"`)
rather than passing silently:

- Non-literal `effect` (such as `var.effect`) or non-list `resources` (such as
  `var.resources`)
- Non-literal entries in `resources = [...]` whose literal segments have no
  wildcard (such as `"arn:aws:s3:::example.invalid/receipts/${var.file}"` or
  `var.arn`)
- `dynamic "statement"` blocks and merged policy attributes
  (`source_policy_documents`, `override_policy_documents`, `source_json`,
  `override_json`)
- IAM policy resources (`aws_iam_policy`, `aws_iam_role_policy`,
  `aws_iam_user_policy`, `aws_iam_group_policy`, and `aws_iam_role`
  `inline_policy`) whose `policy` attribute is not a reference to a
  `data "aws_iam_policy_document"` declared in the same file (for example
  `jsonencode(...)` or an external data source)
- Selected `.tf.json` files

The check does not run Terraform or evaluate the effective deployed policy.
Unreadable or malformed selected `.tf` files appear in `incomplete`. Other
files continue to produce findings.

## Valid example

```hcl
// infra/policies/receipt.tf
data "aws_iam_policy_document" "receipt_reader" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::example.invalid/receipts/current.json"]
  }
}
```

## Counterexample

```hcl
// infra/policies/receipt.tf
data "aws_iam_policy_document" "receipt_reader" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["*"]
  }
}
```

## Fix

Replace wildcard entries with explicit resource ARNs. If the action requires a
wildcard, add a suppression beside the `resources` attribute and explain why.

## Suppression

Use `no-mistakes-disable-next-line` immediately before the `resources` attribute
or `no-mistakes-disable-line` on that attribute's opening line. Reserve
`no-mistakes-disable-file` for an intentional whole-file exemption.

```hcl
data "aws_iam_policy_document" "bucket_catalog" {
  statement {
    actions = ["s3:ListAllMyBuckets"]
    # no-mistakes-disable-next-line terraform-iam-no-undocumented-wildcards: This action requires all resources.
    resources = ["*"]
  }
}
```

Run `no-mistakes check --root . --format json` or the async Node `check()` API.
Add `--include-suppressed` in the CLI, or `includeSuppressed: true` in Node, to
inspect the scoped exceptions.

## Related rules

[`structured-config-policy`](structured-config-policy.md) checks declared key
and value shapes in JSON and YAML configuration.
