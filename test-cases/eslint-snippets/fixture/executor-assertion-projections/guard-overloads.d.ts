declare function ensure(value: unknown, status?: number, message?: string): asserts value;
declare function ensure(
  value: unknown,
  status?: number,
  properties?: Record<string, unknown>,
): asserts value;

declare namespace ensure {
  type Detail = Record<string, unknown>;
  const equal: (actual: unknown, expected: unknown) => void;
}

export = ensure;
