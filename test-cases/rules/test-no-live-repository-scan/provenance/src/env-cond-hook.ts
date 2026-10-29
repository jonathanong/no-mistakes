import { readdirSync } from "node:fs";

const getDir = () => process.env.COND_HOOK_DIR ?? "./tmp/cond-hook";

export function listCondHook() {
  return readdirSync(getDir());
}
