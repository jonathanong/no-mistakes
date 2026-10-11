import { scalarPayload } from "./scalar.mjs";

export function callbackPayload(flag: boolean, statement: string) {
  const scalars = scalarPayload(flag);
  // The cycle and captured Arguments identity must survive branch compaction.
  const cycle = (arguments[1000] = arguments);
  const callback0 = () =>
    flag
      ? [arguments[1], scalars, ["callback-0", "active"]]
      : [arguments, scalars, ["callback-0", "fallback"]];
  const callback1 = () =>
    flag
      ? [arguments[1], scalars, ["callback-1", "active"]]
      : [arguments, scalars, ["callback-1", "fallback"]];
  const selected = flag ? callback0 : callback1;
  return [callback0, callback1, selected, cycle];
}
