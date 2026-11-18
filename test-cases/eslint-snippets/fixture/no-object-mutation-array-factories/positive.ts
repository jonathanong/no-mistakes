declare function consume(value: unknown): void;
export function entries(groups: Record<string, number>) {
  const entries = Object.entries(groups);
  const ready: number[] = [];
  entries.map(([, value]) => { ready.push(value); return value; });
  consume(ready);
}
export function castEntries(groups: Record<string, number>) {
  const entries = Object.entries(groups) as Array<[string, number]>;
  const ready: number[] = [];
  entries.forEach(([, value]) => { ready.push(value); });
  return ready;
}
export function satisfiedEntries(groups: Record<string, number>) {
  const entries = Object.entries(groups) satisfies Array<[string, number]>;
  const ready: number[] = [];
  entries.flatMap(([, value]) => { ready.push(value); return [value]; });
  return ready;
}
export function keys(groups: Record<string, number>) {
  const ready: string[] = [];
  Object.keys(groups).filter(key => { ready.push(key); return true; });
  return ready;
}
export function values(groups: Record<string, number>) {
  const ready: number[] = [];
  Object.values(groups).forEach(value => { ready.push(value); });
  return ready;
}
export function from(input: Iterable<number>) {
  const ready: number[] = [];
  Array.from(input).forEach(value => { ready.push(value); });
  return ready;
}
export function literal() {
  const ready: number[] = [];
  ([1, 2] as number[]).map(value => { ready.push(value); return value; });
  return ready;
}
export function fromMapper(input: Iterable<number>) {
  const ready: number[] = [];
  const rows = Array.from(input, value => value * 2) satisfies number[];
  rows.forEach(value => { ready.push(value); });
  return ready;
}
