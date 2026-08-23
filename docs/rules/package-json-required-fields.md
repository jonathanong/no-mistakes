# `package-json-required-fields`

Flags `package.json` manifests that do not match configured field-shape
policy: `private`, `type`, `license`, scoped `name`, and `main` when a
companion entry file exists. Application-specific name pins stay local.

Empty options report nothing. Each check is opt-in.

```yaml
rules:
  - rule: package-json-required-fields
    scope: repository
    options:
      private: true
      type: module
      license: UNLICENSED
      requireScopedName: true
      unscopedNameExceptions: [web]
      mainWhenFileExists: index.mts
```

Counterexample: an unscoped workspace package missing required fields.

```json
{
  "name": "foo"
}
```

Fix: declare the configured shape.

```json
{
  "name": "@scope/foo",
  "private": true,
  "type": "module",
  "license": "UNLICENSED",
  "main": "index.mts"
}
```

Use `no-mistakes-disable-file package-json-required-fields` when a whole
manifest is an intentional exception.
