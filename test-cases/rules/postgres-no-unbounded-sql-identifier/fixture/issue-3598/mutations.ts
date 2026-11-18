import { read } from "@example/db";
declare function escape(value: unknown): void;
declare const external: string;
const WRITTEN = ["painted", "plain"] as const;
type Written = (typeof WRITTEN)[number];
(WRITTEN as unknown as string[]).push(external);
export function mutated(kind: Written) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
const ESCAPED = ["painted", "plain"] as const;
type Escaped = (typeof ESCAPED)[number];
escape(ESCAPED);
export function escaped(kind: Escaped) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
