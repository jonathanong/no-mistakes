const prototype = Array.prototype;
export function overridden() {
  prototype.map = callback => callback(1);
  const ready: number[] = [];
  [1].map(value => { ready.push(value); return value; });
}
