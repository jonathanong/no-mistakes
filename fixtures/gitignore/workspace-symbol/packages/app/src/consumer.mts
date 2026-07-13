import { hiddenRun } from "@fixture/hidden";
import { visibleRun } from "@fixture/visible";

export function execute() {
  return hiddenRun() + visibleRun();
}
