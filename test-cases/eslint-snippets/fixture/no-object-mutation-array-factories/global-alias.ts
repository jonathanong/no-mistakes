declare function replacement(value: unknown): any;
const Objects = Object;
const Arrays = Array;
export function objectAlias(groups: Record<string, number>) {
  Objects.entries = replacement;
  const ready: unknown[] = [];
  Object.entries(groups).forEach(value => { ready.push(value); });
}
export function arrayAlias(input: Iterable<number>) {
  Arrays.from = replacement;
  const ready: unknown[] = [];
  Array.from(input).forEach(value => { ready.push(value); });
}
