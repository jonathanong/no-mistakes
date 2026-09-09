import { first } from "./first.mts";

export function second() {
  setTimeout(() => {}, 1);
  first();
}
