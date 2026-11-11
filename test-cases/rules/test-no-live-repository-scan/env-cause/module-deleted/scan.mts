import { readdirSync } from "node:fs";

const dir = process.env.EXAMPLE_LOCAL_DIR ?? "./tmp/example";
export const files = readdirSync(dir);
