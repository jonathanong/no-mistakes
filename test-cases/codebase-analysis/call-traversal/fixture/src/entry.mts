import { diamondLeft } from "./diamond-left.mts";
import { diamondRight } from "./diamond-right.mts";
import { cycleA } from "./cycle-a.mts";

export function entry() {
  diamondLeft();
  diamondRight();
  cycleA();
}

entry();
