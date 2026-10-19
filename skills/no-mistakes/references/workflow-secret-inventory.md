# Workflow secret inventory

Use the `workflow-topology-policy` rule's `secretInventory` option to check
GitHub Actions secret names against declared repository and environment names.
The option is opt-in: omit it or set it to `null` to disable the check; an
empty object enables a strict empty inventory.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      secretInventory:
        repository: [CI_DEPLOY_TOKEN]
        environments:
          production: [RELEASE_KEY]
        reportUnused: true
```

Environment names and secret names match without ASCII case sensitivity. A
literal job environment selects its environment inventory, then falls back to
repository declarations; environment entries shadow a same-named repository
secret. List organization secrets and Dependabot secrets under `repository`.
Workflow-level `env` is checked against `repository`. A missing name is
undeclared. The same name is not reported unused in an environment inventory,
because workflow-level `env` versus job `environment` is not verified. Reusable
workflow secret aliases are contracts, so inventory the caller's actual source
name. A static name with no contract alias is still undeclared when it is
absent from the repository and every environment inventory. A static `secrets:`
map on `owner/repo/...@ref` is fully known for undeclared names. The built-in
`GITHUB_TOKEN` exemption applies unless `builtInExemptions: []` is configured.

Use `no-mistakes check --root <repo> --format json` to inspect findings. Dynamic
selectors (including double-quoted `secrets["NAME"]`), inherited secrets,
expression-valued environments, unresolved reusable calls, and workflow load
failures make unused-name analysis incomplete. Incomplete findings are emitted
only when `reportUnused` is true; they do not hide a static undeclared name.
Do not treat an incomplete run as proof that a declared name is unused.
Composite actions cannot access `secrets.*` directly; pass the value through an
input or environment variable. Only actions reached through `uses: ./...`
(including `uses: ./` for the repository root) are scanned. Actions used only
by remote callers, or invoked by a path in a script, are not scanned. Job and
step `if:` keys are bare expressions; `with.if` is not.

See the [full configuration guide](https://github.com/jonathanong/no-mistakes/blob/main/docs/configuration/workflow-topology-policy.md)
and the [rule reference](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/workflow-topology-policy.md).
