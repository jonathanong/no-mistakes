import { invoke00 } from "./helper-00.mjs";
type QueryCallback = () => unknown;
export function invoke01(callback: QueryCallback, payload: unknown) {
  return invoke00(callback, payload);
}
