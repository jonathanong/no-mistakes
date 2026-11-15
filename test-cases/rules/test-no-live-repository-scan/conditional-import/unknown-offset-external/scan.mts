import { readdirSync } from "node:fs";
export const loaded = readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
