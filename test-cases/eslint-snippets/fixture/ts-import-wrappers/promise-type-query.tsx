import * as React from "react";
const P = Promise as typeof Promise;
export function cast() {
  return React.use(P.resolve(1));
}

const Q = Promise as typeof globalThis.Promise;
export function query() {
  return React.use(Q.resolve(1));
}
