# `terraform-iam-no-undocumented-wildcards`

## Why and when

A wildcard in an IAM Allow statement can grant access to more resources than
the caller needs. Some AWS actions require a wildcard. Record why the action
needs it where reviewers can see the scope of the grant.
[AWS IAM resource reference](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_resource.html)

## Options and defaults

The rule is off until configured. It has no rule-specific options. Use standard
`projects` or `scope`, and `include` and `exclude`, to select Terraform or
OpenTofu files that use the AWS provider's `aws_iam_policy_document` data source.

```yaml
rules:
  - rule: terraform-iam-no-undocumented-wildcards
    scope: repository
    include: ["infra/policies/**/*.tf"]
```

## What it catches

The rule parses selected native `.tf` files and visits direct `statement`
blocks in `data "aws_iam_policy_document"` declarations. It reports a
`resources` attribute when a literal array entry contains a wildcard (`*`
or `?`) and the effect is explicitly `Allow` or defaults to `Allow`. It
reports once per attribute, even when several entries contain wildcards.
AWS policy variable expressions such as `&{*}` can denote literal characters
and do not count as wildcards.
[AWS IAM policy variables](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_variables.html#policy-vars-specialchars)

The check covers source declarations. It skips Deny statements, dynamic
`statement` blocks, computed effects and resource values, `not_resources`,
JSON policies, and policies loaded or merged from other documents.
It does not run Terraform or evaluate the effective deployed policy.
Unreadable or malformed selected files appear in `incomplete`. Other files
continue to produce findings.

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
