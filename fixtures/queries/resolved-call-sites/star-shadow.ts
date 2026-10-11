export * from "./target";
// This value export owns `used`, so it shadows the star-forwarded original.
export function used(value: unknown) { return value; }
