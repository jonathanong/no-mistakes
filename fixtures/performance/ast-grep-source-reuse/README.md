# Ast-grep source reuse fixtures

This saved corpus covers one app, 12 apps, and 20 apps. Each app has ten rule
YAML files plus its app config. The cases provide repeatable inputs for checking ast-grep freshness
behavior and source/structured-YAML reuse.

These fixtures establish semantics and input shape only. They are not
measurements and make no performance claim. Parent PR measurement results must
report the exact benchmark scope and before/after wall time, CPU time, and peak
RSS.
