// Flagged: exported, extends Error, never constructed or subclassed.
export class UnusedError extends Error {}

// Constructed in src/use.ts.
export class ConstructedError extends Error {}

// Flagged: the only construction is in a test file.
export class TestOnlyError extends Error {}

// Flagged: built-in subclass, never constructed.
export class UnusedRangeError extends RangeError {}

// Flagged: `globalThis.Error` is still the built-in Error.
export class UnusedGlobalError extends globalThis.Error {}

// Constructed by the function below, in the same file.
export class SameFileError extends Error {}

export function failInFile(): never {
  throw new SameFileError("same file");
}

// Ignored: not exported.
class PrivateUnusedError extends Error {}
void PrivateUnusedError;

// Ignored: not an error class.
export class NotAnError {}

// Ignored: extends a class that is not an Error.
export class ExtendsNotAnError extends NotAnError {}

// Flagged: exported through a clause.
class ClauseUnusedError extends Error {}
export { ClauseUnusedError };

// Flagged: exported under another name.
class RenamedExportError extends Error {}
export { RenamedExportError as PublicName };

// Flagged: a default export is still an exported class.
export default class DefaultUnusedError extends Error {}
