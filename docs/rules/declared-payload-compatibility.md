# `declared-payload-compatibility`

## Why and when

Checks explicit producer and consumer JSON Schema declarations directionally:
every value permitted by the producer must be accepted by the consumer. This
helps independently compiled HTTP clients/servers and queue producers/workers
keep their declared payloads compatible. It checks declarations only; it does
not prove endpoints use those declarations or infer actual runtime payloads.

## What it catches

Reports renamed or optional producer fields required by a consumer, narrowed
consumer enums, nested type drift, and incompatible object/array declarations.
Unsupported forms cannot establish compatibility and are reported as unproven.

## Configuration

```yaml
rules:
  - rule: declared-payload-compatibility
    scope: repository
    options:
      contracts:
        - name: queue user-notifications
          producer: {file: api/payloads.json, pointer: /notification}
          consumer: {file: worker/payloads.json, pointer: /notification}
        - name: HTTP request POST /users
          producer: {file: client/request.json}
          consumer: {file: server/request.json}
        - name: HTTP response GET /users
          producer: {file: server/response.json}
          consumer: {file: client/response.json}
```

`contracts` defaults to empty, so analysis is opt-in. Every contract requires a
nonempty `name`, `producer`, and `consumer`. Each reference requires `file` and
may specify `pointer`. Unknown option fields and malformed option shapes are
configuration errors. Schema problems produce findings labeled `unproven`;
known declaration incompatibilities produce findings labeled `incompatible`.
Both cause `check` to fail. Findings identify the contract, document, and nested
payload property. The normal rule application `message` prefixes the diagnostic, preserving its
incompatible/unproven classification.

Files are relative to the invocation root, including project-scoped rules, and
must belong to the prepared visible inventory and this rule's scope. Missing,
ignored, excluded, unreadable, or out-of-scope documents produce `unproven`
findings. Paths with `.`/`..` components and absolute paths are rejected. No
directory conventions, endpoint matching, or global scans are inferred.

Pointers follow RFC 6901: empty selects the document, `/notification` selects
that member, and `/a~1b~0c` selects the member named `a/b~c`. The pointed value
must be a supported schema. Wrapper keys outside the selected schema are not
interpreted; selecting a definition does not resolve its references.

## Valid example, counterexample, and fix

This producer and consumer are compatible:

```json
{
  "type": "object",
  "properties": {"userId": {"type": "string"}},
  "required": ["userId"],
  "additionalProperties": false
}
```

Renaming only the producer property to `accountId` is a counterexample:

```json
{
  "type": "object",
  "properties": {"accountId": {"type": "string"}},
  "required": ["accountId"],
  "additionalProperties": false
}
```

The consumer still requires `userId`, which the producer no longer guarantees.
Fix both declarations and their implementations together, or preserve the old
field until the consumer migration finishes. Making the producer field optional
also fails when the consumer requires it. A response contract reverses the
application roles: the server produces and the client consumes.

## Supported schema subset and limits

Every schema node must declare one string `type`: `object`, `array`, `string`,
`integer`, `number`, `boolean`, or `null`. Supported constraints are:

- Objects: `properties`, `required`, and boolean `additionalProperties`.
  Required names must have declared property schemas. Optional properties are
  checked too. `additionalProperties` defaults to `true` as in JSON Schema;
  an open producer cannot satisfy a closed consumer. An open producer also
  fails if an undeclared producer property has a constrained consumer schema.
- Arrays: a single supported `items` schema is required and checked recursively.
- Primitive values: optional nonempty `enum` or `const`. Values must match their
  declared type; duplicate enum values are rejected. When the consumer constrains
  values, producer values must be a subset. Unrestricted boolean and null types
  are finite; other unrestricted primitive producers require compatible
  unrestricted consumer types.
- String `title`, `description`, and `$comment` annotations do not restrict values.

Integer producers can satisfy number consumers. Number producers satisfy
integer consumers only when an enum/const proves every value integral. Numeric
equality treats `1` and `1.0` equally and preserves distinct large integer values.
Every raw numeric token in the JSON document must retain its exact decimal
value after parsing; truncation such as `1.0000000000000001` is `unproven`.
Duplicate JSON object keys anywhere in the document are also rejected. These
document integrity checks include wrapper metadata outside the selected schema.
Floating enum/const values with magnitude at least 2^53 are `unproven` to avoid
loss of precision. Nesting beyond 64 levels is `unproven`.

All other keywords/forms are `unproven`, including `$schema`, `$id`, `$ref`,
`$defs` inside the selected node, type unions, boolean schemas, combinations,
tuple items, schema-valued additional properties, bounds, patterns, and formats.
Even identical unsupported schemas fail. This deliberately bounded subset is
not a general JSON Schema validator or a TypeScript type checker. A clean result
means the selected supported declarations are compatible, not that runtime
payloads have been validated.

## CLI and Node API

```bash
no-mistakes check --root . --format json
```

```js
const {check} = require("no-mistakes");
const report = await check({root: ".", config: ".no-mistakes.yml"});
```

The existing asynchronous `check()` and `analyzeProject()` check report expose
the same `RuleFinding` entries. Named configuration types are exported as
`DeclaredPayloadCompatibilityOptions`, `DeclaredPayloadContract`, and
`DeclaredPayloadSchema`.

## Suppression

Findings use the first nonempty line of the referenced document after leading
`//` comments; the nested payload path is included in the diagnostic. The shared
`no-mistakes-disable-file`, `no-mistakes-disable-line`, and
`no-mistakes-disable-next-line` directives apply to findings at those locations.
A top-of-file comment can suppress an intentionally unavailable declaration;
JSON with such comments is not a supported schema and would otherwise be
`unproven`. A JSON string containing a directive is data, not a suppression
comment. For valid JSON declarations, disable the rule application or remove the
explicit pair when appropriate. Excluding a declared schema produces an
`unproven` finding rather than silently skipping a contract.

## Related rules

- [finite-set-consistency](finite-set-consistency.md) compares finite strings
  extracted from different file formats.
- [structured-config-policy](structured-config-policy.md) checks configuration
  keys and explicit value assertions.
