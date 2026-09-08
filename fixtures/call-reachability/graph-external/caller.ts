import { api, fn, starFn, value } from "./barrel";

export function run() {
  fn();
  api.member();
  starFn();
  // Explicit value reads still do not enter call-only traversal.
  return value;
}
