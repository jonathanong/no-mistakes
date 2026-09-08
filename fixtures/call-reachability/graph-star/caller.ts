import { api, direct, value } from "./barrel";

export function run() {
  direct();
  api.namespaced();
  // Merely reading an imported value must not widen call-only traversal.
  return value;
}
