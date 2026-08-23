import { linkedSymbol } from "@linked/symbol-target";
import * as linked from "@linked/symbol-target";

export function executeWithoutVisibleTarget(): string {
  return linkedSymbol() + linked.linkedSymbol();
}
