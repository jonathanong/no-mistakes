import { child } from "./child.mjs";
import { excluded } from "./excluded.mjs";

export function entry() {
  setTimeout(() => {}, 1);
  setTimeout(() => {}, 2);
  child();
  excluded();
  excluded();
}
