import { label } from "@scope/pkg";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { value } from "../config.mts";

void label;
void value;

export function listFiles(root: string) {
  readdirSync(join(root, label));
  readdirSync(join(root, value));
}
