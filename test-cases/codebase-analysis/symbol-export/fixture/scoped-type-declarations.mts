import type { SourceShape } from "./source.mts";

type Alias<SourceShape> = SourceShape;

interface Box<SourceShape> {
  value: SourceShape;
}

export function aliasRun(value: Alias<string>) {
  return value;
}

export function interfaceRun(value: Box<string>) {
  return value.value;
}
