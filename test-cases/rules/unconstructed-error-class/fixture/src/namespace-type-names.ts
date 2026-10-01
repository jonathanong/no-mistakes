// Each of these names a namespace in erased code only, which is no use of it
// at run time: a type query, an `implements` clause, and an interface base.
export namespace Queried {
  export class QueriedDead extends Error {}
}
export type Constructors = typeof Queried;

export namespace Marked {
  export interface Marker {}
  export class MarkedDead extends Error {}
}
export class Marks implements Marked.Marker {}

export namespace Extended {
  export interface Base {}
  export class ExtendedDead extends Error {}
}
export interface Shape extends Extended.Base {}
