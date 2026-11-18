declare function replacement(value: unknown): any;
export function objectOverride(groups: Record<string, number>) {
  Object.entries = replacement;
  const ready: unknown[] = [];
  Object.entries(groups).forEach(value => { ready.push(value); });
}
export function arrayOverride(input: Iterable<number>) {
  Array.from = replacement;
  const ready: unknown[] = [];
  Array.from(input).forEach(value => { ready.push(value); });
}
