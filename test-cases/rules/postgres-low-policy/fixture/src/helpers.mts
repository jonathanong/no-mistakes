export function helper() {
  return "SELECT * FROM account"; // PHYSICAL_STAR
}
export function chain() { return helper(); }
declare function unknownSql(): string;
export function partial(flag: boolean) {
  // One feasible literal survives uncertainty about the other branch.
  if (flag) return "SELECT * FROM account";
  return unknownSql();
}
export function invalid() {
  // Keep the parsed first statement when the following SQL is invalid.
  return "SELECT * FROM account; SELECT FROM";
}
