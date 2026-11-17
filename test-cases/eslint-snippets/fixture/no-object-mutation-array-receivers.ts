export class Base extends Array<number> {}
export class Child extends Base {
  update() { this.push(1); this.reverse(); const push = this.push; push.call(this, 2); }
  static custom() { this.push(); }
}
export const ExpressionBase = class extends Array<number> {};
export class ExpressionChild extends ExpressionBase {
  update() { this.pop(); }
}
export function localClasses() {
  class LocalBase extends Array<number> {}
  class LocalChild extends LocalBase {
    update() { this.sort(); }
  }
  return LocalChild;
}
export function tuple(this: [number, number]) { this.reverse(); }
export function globalArray(this: Array<number>) { this.push(1); }
export function valueShadow() {
  const Array = 1;
  return function globalType(this: Array<number>) { this.pop(); };
}
export function generic<Array extends { reverse(): void }>(this: Array) { this.reverse(); }
