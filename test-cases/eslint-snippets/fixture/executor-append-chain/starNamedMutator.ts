import { STAR_NAMED_MUT } from "./starBarrel";

export function extend(extra: unknown) {
  STAR_NAMED_MUT.append(extra);
}
