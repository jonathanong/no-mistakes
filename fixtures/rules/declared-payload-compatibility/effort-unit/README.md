# Payload effort boundary fixtures

`duplicate.json`, `numeric.json`, and `malformed.json` are intentionally invalid
inputs. Preserve the duplicate key, raw numeric token, and malformed syntax so
integrity checks cannot be bypassed by the low ignore policy.

`file.json`, `line.json`, and `next.json` intentionally contain real suppression
comments, which are not valid JSON. They test suppression of the resulting hard
document findings. Directive-looking strings in `schemas.json` are data.

`consumer-nested.json` keeps a separate consumer document so uncertainty must
identify the consumer rather than an otherwise flat producer.
