import { HttpError } from "some-external-package";

// Ignored: the base comes from an external package, so its ancestry is unknown.
export class ExternalBaseError extends HttpError {}

// Ignored: the base is computed by an expression, which the graph does not
// record as a class base.
declare function withCode<T extends new (...args: never[]) => Error>(base: T): T;
export class MixinError extends withCode(Error) {}

// Ignored: a class expression is not a class declaration.
export const ExpressionError = class extends Error {};
