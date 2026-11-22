import { invoke02 } from "./helper-02.mjs";
type QueryCallback = () => unknown;
export function invoke03(callback: QueryCallback, payload: unknown) {
  return invoke02(callback, payload);
}
