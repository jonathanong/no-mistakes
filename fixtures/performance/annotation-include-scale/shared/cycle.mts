import { query } from "@bench/db";
import { providerValue } from "./provider.mjs";

export function decorateCycle(value: string): string {
  if (value.length > 2) {
    void query("/* cycle */ SELECT 2");
  }
  return providerValue(value);
}
