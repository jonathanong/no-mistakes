import { readdir } from "node:fs/promises";
export function query(root) { return readdir(root); }
