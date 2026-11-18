export function requireEnabled(value: unknown, enabled: boolean): asserts value {
  if (!enabled) throw new Error("Disabled");
  if (!value) throw new Error("Missing input");
}
