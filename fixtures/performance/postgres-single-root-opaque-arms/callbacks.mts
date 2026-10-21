import { scalar } from "./scalar.mjs";

export function callbacks(flag: boolean, statement: string) {
  const values = scalar(flag);
  // Preserve both the argument cycle and callbacks over different sibling states.
  const cycle = (arguments[1000] = arguments);
  const first = () => flag ? [arguments[1], values] : [arguments, values];
  const second = () => flag ? [arguments[1], values] : [arguments, values];
  const chosen = flag ? first : second;
  return [first, second, chosen, cycle];
}
