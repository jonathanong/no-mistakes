# Dependency check benchmark

Twenty-four packages form a ring. Each imports the next package and reexports
a local source, so every package has an external production seed and an internal
dependency to traverse. Package p12 deliberately declares its runtime import
only in devDependencies, producing one stable finding.

Three independent forbidden-closure applications read the same pnpm lockfile.
Their roots differ, while the immutable importer projection is identical.
Each benchmark iteration prepares a fresh request to include discovery,
source preparation, import facts, dependency resolution, and rule evaluation.
