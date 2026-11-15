import { readdirSync } from "node:fs";
export function scan() {
  return readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
}
