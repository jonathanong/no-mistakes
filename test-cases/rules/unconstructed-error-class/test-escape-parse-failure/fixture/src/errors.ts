// No file builds `Dead`. The one use of `Errors` that would keep it quiet is
// the alias in `__tests__/alias.ts`, which the parser cannot read.
export namespace Errors {
  export class Dead extends Error {}
}
