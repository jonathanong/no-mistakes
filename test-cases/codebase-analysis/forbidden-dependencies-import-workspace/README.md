This fixture protects import-oriented `forbidden-dependencies` policies.

The entrypoint reaches one forbidden module through a direct import and one
forbidden file through a local workspace package. A literal resource read points
at a third forbidden file, but that edge must stay excluded because the rule
selects only `import` and `workspace` relationships.
