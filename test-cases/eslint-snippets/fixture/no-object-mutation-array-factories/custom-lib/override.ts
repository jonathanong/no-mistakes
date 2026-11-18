export {};
declare global {
  interface Array<T> {
    map(callback: (value: T) => T): T[];
  }
}
export function declaredOverride(rows: number[]) {
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
  return ready;
}
