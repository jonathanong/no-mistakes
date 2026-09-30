import { HttpError } from "some-external-package";

// Ignored: the base comes from an external package, so its ancestry is unknown.
export class ExternalBaseError extends HttpError {}

// Ignored: the base is computed by an expression, which the graph does not
// record as a class base.
declare function withCode<T extends new (...args: never[]) => Error>(base: T): T;
export class MixinError extends withCode(Error) {}

// Ignored: a class expression is not a class declaration.
export const ExpressionError = class extends Error {};

// Not repository classes: building a global or an external-package class
// credits no error class in this repository.
export const registry = new Map<string, string>();
export const failure = new HttpError("boom");
