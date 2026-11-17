type Array<T> = { push(value: T): void };
export function custom(this: Array<number>) { this.push(1); }
export function generic<Array extends { reverse(): void }>(this: Array) { this.reverse(); }
export class NamedArray {
  push() {}
  update() { this.push(); }
}
export function localType() {
  type Array<T> = { pop(): T };
  return function custom(this: Array<number>) { this.pop(); };
}
export function localClass() {
  class Array<T> { push(value: T) {} }
  class Child extends Array<number> { update() { this.push(1); } }
  return function custom(this: Array<number>) { this.push(1); };
}
export function localInterface() {
  interface Array<T> { pop(): T }
  return function custom(this: Array<number>) { this.pop(); };
}
