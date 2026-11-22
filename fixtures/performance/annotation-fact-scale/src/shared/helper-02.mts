import { invoke01 } from "./helper-01.mjs";
type QueryCallback = () => unknown;
export function invoke02(callback: QueryCallback, payload: unknown) {
  return invoke01(callback, payload);
}
