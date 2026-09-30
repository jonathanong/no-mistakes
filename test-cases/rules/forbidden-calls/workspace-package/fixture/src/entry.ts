import { danger, safe } from "@fixture/lib";
import { subDanger } from "@fixture/lib/sub";

export function run() {
  safe();
  danger();
  subDanger();
}
