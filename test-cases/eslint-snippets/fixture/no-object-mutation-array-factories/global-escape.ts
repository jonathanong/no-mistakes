declare function consume(value: unknown): void;
const Objects = Object;
const Arrays = Array;
export function objectEscape(groups: Record<string, number>) {
  consume(Objects);
  const ready: unknown[] = [];
  Object.entries(groups).forEach(value => { ready.push(value); });
}
export function arrayEscape(input: Iterable<number>) {
  consume(Arrays);
  const ready: unknown[] = [];
  Array.from(input).forEach(value => { ready.push(value); });
}
