# Shared package construction proof

The pg and Ajv declarations intentionally supply complete class ancestry here.
The older `real-imports` and `construction-targets` fixtures retain their opaque
controls. The package names below exercise generic declaration shapes, rather
than package-specific constructor exemptions.

`Pool` follows a default object declaration, destructuring, a local export, and a
source alias facade. `AjvCtor` follows a nullish default/module alternative.
The worker specifier is explicitly mapped to a visible declaration, so its name
must never be treated as intrinsic proof. Error subclasses protect exact class
identity, while opaque and unavailable packages must continue to advise.

The Configured constructor passes through an exported typed constant that narrows
constructor options, then a second alias barrel; Filter follows the same two-hop
barrel path without a typed constant. Both originate in generic package classes.
Mutation controls distinguish a write before a constructor snapshot (uncertain)
from a write after it (the immutable snapshot retains its class identity).
