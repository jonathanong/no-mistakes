import { fetch as renamed } from "client";
declare const factory: () => <T>() => T;

export function extra() {
  factory()<string>;
  const f = factory()<string>;
  const value = renamed("/a");
}
