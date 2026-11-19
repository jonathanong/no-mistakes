import * as parts from "./starBarrel";

export function extend(extra: unknown) {
  parts.STAR_NS_MUT.append(extra);
}
