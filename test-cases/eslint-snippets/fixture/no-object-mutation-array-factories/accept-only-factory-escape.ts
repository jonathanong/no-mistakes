declare function consume(value: unknown): void;
export function mutate(groups: Record<string, number>, count: number) {
  const entries = Object.entries(groups);
  consume(entries);
  entries.push(["extra", 1]);
  const keys = Object.keys(groups);
  consume(keys);
  keys.push("extra");
  const mapped = Array.from({ length: count }, () => ({}));
  consume(mapped);
  mapped.push({});
}
