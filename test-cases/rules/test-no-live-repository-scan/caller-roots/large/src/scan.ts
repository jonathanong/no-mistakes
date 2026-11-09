import * as fs from "node:fs";
import { join } from "node:path";

export function scan(root: string) {
  return fs.readdirSync(join(root, "items"));
}

export function scanOptions(options: { root: string }) {
  return fs.readdirSync(join(options.root, "items"));
}

export function scanEnv() {
  return fs.readdirSync(join(process.env.SCAN_ROOT ?? "", "items"));
}

export function scanForwarded(root: string) {
  return fs.readdirSync(join(root, "items"));
}

export function scanLiteral(root: string) {
  return fs.readdirSync(join(root, "items"));
}

export function scanMixed(root: string) {
  return fs.readdirSync(join(root, "items"));
}

export function scanUnknown(root: string) {
  return fs.readdirSync(join(root, "items"));
}

export function scanMutable(root: string) {
  return fs.readdirSync(join(root, "items"));
}
