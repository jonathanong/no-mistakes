export function requireValue(value: unknown): asserts value {
  if (!value) throw new Error("Input required");
}

export function writeInAssertion(value: unknown): asserts value {
  if (!value) throw new Error("Input required");
  (value as { id: string }).id = "changed";
}

declare function register(callback: () => unknown): void;

export function captureInAssertion(value: unknown): asserts value {
  if (!value) throw new Error("Input required");
  register(() => value);
}

export function replaced(value: unknown): asserts value {
  if (!value) throw new Error("Input required");
}

replaced = writeInAssertion;

export function assertFound(value: unknown, status?: number, message?: string): asserts value {
  if (!value) throw new Error("Notice required");
}
