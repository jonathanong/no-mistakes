export function assertIdentifier(value: string): void {
  if (!["public", "users"].includes(value)) throw new Error("Unknown SQL identifier");
}
