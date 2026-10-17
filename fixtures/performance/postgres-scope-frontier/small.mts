// Uncalled inert bindings expose repeated module-scope liveness scans.
export const statement = "/* frontier */ SELECT 1";
export function read0() {
  return statement;
}
export function read1() {
  return statement;
}
export function read2() {
  return statement;
}
export function read3() {
  return statement;
}
