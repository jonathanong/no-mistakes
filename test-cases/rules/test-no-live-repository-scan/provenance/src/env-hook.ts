import { readdir } from "node:fs/promises";

const getDir = () => process.env["HOOK_DIR"] || "./tmp/hook";

export async function listHook() {
  return readdir(getDir());
}
