// Every global is exported, and the references form a cycle (left -> right ->
// middle -> left). Fully exported modules must take the All shortcut and must
// not walk that cycle. The re-export is not a local global.
export function left() {
  return right;
}
export function right() {
  return [left, middle];
}
export const middle = left;
export { entry } from "./export-reachability-single-export";
