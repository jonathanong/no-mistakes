import { readdirSync } from "node:fs";

export function syncReassigned(dir: string) {
  dir = process.env.PARAM_REASSIGNED ?? dir;
  return readdirSync(dir);
}
