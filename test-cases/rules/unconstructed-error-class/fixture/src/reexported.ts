// Constructed in src/use.ts through an import alias.
export class AliasedError extends Error {}

// Constructed in src/use.ts through a named re-export.
export class BarrelError extends Error {}

// Constructed in src/use.ts through `export *`.
export class StarError extends Error {}

// Constructed in src/use.ts through a namespace import.
export class NamespaceError extends Error {}

// Flagged: the barrel re-exports it but nothing constructs it.
export class BarrelUnusedError extends Error {}
