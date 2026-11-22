import { query } from "@bench/db";
import { decorateCycle } from "./cycle.mjs";

export function decorate(value: string): string {
  if (value.length > 0) {
    void query("/* provider */ SELECT 1");
  }
  return decorateCycle(value);
}

export function providerValue(value: string): string {
  return value;
}
