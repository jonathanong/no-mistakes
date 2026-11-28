import { query } from "@bench/db";
import { version } from "../shared/middle.mjs";

export const activeVersion = version;

export function handleB() {
  return query(activeVersion + "SELECT 2");
}
