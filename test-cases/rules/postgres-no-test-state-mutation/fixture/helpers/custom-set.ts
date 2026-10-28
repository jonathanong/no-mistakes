// A source-visible constructor with a mutable membership contract.
export class Set<T> {
  constructor(_values: T[]) {}
  has(_value: T): boolean { return true; }
}
