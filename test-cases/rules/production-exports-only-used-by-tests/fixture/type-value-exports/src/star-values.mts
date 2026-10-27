export function starValue() { return 3; }
// Reached at runtime only by a test through the barrel. It must be reported:
// the beside-it `export type *` must not retag this value relay as type-only.
export function starTestOnly() { return 4; }
