# Low SQL executor transport controls

These fixtures distinguish creation and transport of executor-bearing values
from calls that may execute SQL. A proven global `Object.freeze` call creates a
value without creating a SQL obligation. Its executor identity remains tracked:
direct calls, detached receivers, methods, callback bodies, and opaque escapes
stay uncertain at their actual use. Ordinary metadata reads stay quiet.

The shadowed, mutated, escaped, and `globalThis` cases use separate files because
any such use invalidates the file-wide intrinsic proof. Factory options are
metadata when passed to a configured factory without a callback; the actual
factory result is checked at its call sites. Unsupported destructuring and
opaque factory-result consumers remain unknown.

The policy fixture exercises all four low SQL rules in advise, report, and ignore
modes. The expected call locations are marked in the source. See
[executor binding behavior](../../../../../docs/postgres-facts/executor-bindings.md)
for the configuration contract.
