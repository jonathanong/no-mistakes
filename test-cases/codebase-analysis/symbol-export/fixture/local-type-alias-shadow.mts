import type { SourceShape } from "./source.mts";

export function aliasRun() {
  type SourceShape = string;
  const value: SourceShape = "local";
  return value;
}

export function interfaceRun() {
  interface SourceShape {
    value: string;
  }
  const value: SourceShape = { value: "local" };
  return value;
}

