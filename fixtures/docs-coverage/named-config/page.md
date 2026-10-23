# Named config materialization regression

```yaml
rules:
  - rule: postgres-object-naming
    scope: repository
    options:
      schemaCatalogPath: fallback.json
```

## Named config

```yaml
# .no-mistakes.yml
projects:
  web:
    root: web
rules:
  - rule: postgres-object-naming
    scope: repository
    message: 'uses: is literal text in this config, not workflow syntax'
    options:
      schemaCatalogPath: db/catalog.json
      patterns:
        table: '^approved_'
```

```json
// db/catalog.json
{"formatVersion":2,"coverage":"complete","tables":{"unapproved":{}}}
```

## Unnamed config

```yaml
rules:
  - rule: postgres-object-naming
    scope: repository
    options:
      schemaCatalogPath: ignored.json
```
