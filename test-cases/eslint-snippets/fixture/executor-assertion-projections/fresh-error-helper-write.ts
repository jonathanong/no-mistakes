Object.assign = ((value: unknown) => value) as typeof Object.assign;

export function check(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { status });
}
