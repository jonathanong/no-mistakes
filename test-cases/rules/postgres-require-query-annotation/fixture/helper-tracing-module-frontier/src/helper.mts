// Immutable initial bindings qualify for the module-only frontier.
export const statement = "/* frontier */ SELECT 1";
export const missing = "SELECT missing_frontier_annotation";
export function read() {
  return statement;
}
export function bare() {
  return missing;
}
export function choose(flag: boolean) {
  return flag ? read : read;
}
