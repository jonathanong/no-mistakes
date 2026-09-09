import { selectedTarget } from "./selected-call-target.mts";
import { unrelatedTarget } from "./selected-call-unrelated.mts";

export function selected() {
  selectedTarget();
}

unrelatedTarget();
