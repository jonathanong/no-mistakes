import { readdirSync } from "node:fs";

export const names = readdirSync(process.env.TOP_DIR ?? "./tmp/top");
