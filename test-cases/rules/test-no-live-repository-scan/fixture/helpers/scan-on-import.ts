import { readdirSync } from "node:fs";

export const names = readdirSync("../../.github");
