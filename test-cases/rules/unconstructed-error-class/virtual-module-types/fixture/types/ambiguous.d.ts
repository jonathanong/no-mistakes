// Two classes share one export. A unique class of this module in a types
// package must not replace the ambiguity.
declare module 'fixture:ambiguous' {
  export class Base {}
}
declare module 'fixture:ambiguous' {
  export class Base extends Error {}
}
