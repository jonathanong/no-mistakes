// Not flagged: src/workspace-use.ts constructs it through the package name.
export class WorkspaceError extends Error {}

// Flagged: exported by the package but never constructed.
export class WorkspaceUnusedError extends Error {}

// Not flagged: src/workspace-use.ts extends it through the package name.
export abstract class LibBaseError extends Error {}
