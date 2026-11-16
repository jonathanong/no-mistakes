export function mutate() {
  this.items.push(1);
  this.value = 1;
  Object.assign(this, { value: 1 });
  Array.prototype.push.call(this, 1);
}
export const rows = [];
rows.push(1);
rows.forEach(function () { this.push(1); });
export function knownReceiver(rows) {
  const push = rows.push;
  push.call(this, 1);
}
export function typedArray(this: number[]) { this.push(1); }
export function genericArray(this: Array<number>) { this.pop(); }
export class ArrayChild extends Array {
  append() { this.push(1); }
}
