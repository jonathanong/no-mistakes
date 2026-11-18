export function visible(value: unknown): asserts value;
export function visible(value: unknown, status: number): asserts value;
export function visible(value: unknown, status?: number): asserts value {
  if (!value) throw "Missing";
}

export function writing(value: unknown): asserts value;
export function writing(value: unknown, status: number): asserts value;
export function writing(value: unknown, status?: number): asserts value {
  (value as { id: string }).id = "changed";
}

export declare function mixed(value: unknown): asserts value;
export declare function mixed(value: unknown, status: number): boolean;

export declare function differentIndex(value: unknown, other: unknown): asserts value;
export declare function differentIndex(
  value: unknown,
  other: unknown,
  status: number,
): asserts other;

export function runtimeMerged(value: unknown): asserts value {
  if (!value) throw "Missing";
}

export namespace runtimeMerged {
  export const marker = true;
}
