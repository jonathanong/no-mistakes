import * as parts from "./typeofValueFragments";

const kind = typeof parts;

export function extend(kindName: string) {
  return [kind, kindName];
}
