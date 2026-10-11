import { query } from "@bench/db";
import { scalarPayload } from "./scalar.mjs";
import { callbackPayload } from "./callbacks.mjs";

declare function opaqueConsumer(...values: unknown[]): unknown;

export async function loadRoute(flag: boolean) {
  const scalar = scalarPayload(flag);
  const callbacks = callbackPayload(flag, "/* helper context */ SELECT id FROM orders");
  const combined = flag ? [callbacks, scalar] : [scalar, callbacks];
  opaqueConsumer(combined);
  return [
    query("/* loadRoute */ SELECT id FROM orders"), // safe-query
    query("SELECT status FROM orders"), // unsafe-query
  ];
}
