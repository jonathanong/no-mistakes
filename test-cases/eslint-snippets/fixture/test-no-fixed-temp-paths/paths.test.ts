import path from "node:path";
import * as os from "node:os";
import { mkdtemp as makeDirectory } from "node:fs/promises";
import { mkdtempSync } from "fs";

export const fixed = "/tmp/result.json";
export const fixedSystem = "/var/tmp/result.json";
export const escaped = "\x2ftmp/result.json";
export const staticTemplate = `/tmp/result.json`;
export const joined = path.join(os.tmpdir(), "example", `result.json`);
export const joinedWithCast = path.join(os.tmpdir() as string, "result.json" as const);
export const root = path.join(os.tmpdir());

export const directory = await makeDirectory(path.join(os.tmpdir(), "example-"));
export const literalPrefix = mkdtempSync("/tmp/example-");
export const typedPrefix = mkdtempSync("/var/tmp/example-" as const);
export const result = path.join(directory, "result.json");
export const unrelated = ["/tmp", "/var/tmpfiles/result.json", "fixtures/tmp/result.json"];

export function dynamic(segment: string, segments: string[]) {
  return [
    path.join(os.tmpdir(), segment, "result.json"),
    path.join(os.tmpdir(), `example-${segment}`),
    path.join(os.tmpdir(), ...segments),
    `/tmp/example-${segment}`,
  ];
}
