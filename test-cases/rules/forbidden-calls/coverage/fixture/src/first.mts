import { second } from "./second.mts";

export function first() {
  setTimeout(() => {}, 1);
  second();
}
