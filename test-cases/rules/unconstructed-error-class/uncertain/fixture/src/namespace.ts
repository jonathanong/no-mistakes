export namespace Escaped {
  export class HiddenError extends Error {}
  export class KnownError extends Error {}
}
// Escaping the namespace is uncertain, while an exact construction remains proof.
export const registry = Escaped;
new Escaped.KnownError();
