declare function consume(value: unknown): void;
export function escapedThis(this: number[]) {
  const ready: number[] = [];
  consume(this);
  this.forEach(value => { ready.push(value); });
  return ready;
}
export class Deferred extends Array<number> {
  map(callback: (value: number) => number): number[] {
    setTimeout(() => callback(1), 0);
    return [];
  }
  build() {
    const ready: number[] = [];
    this.map(value => { ready.push(value); return value; });
    return ready;
  }
}
