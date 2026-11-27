import * as React from "react";
const P = Promise as typeof Promise;
export function cast() {
  return React.use(P.resolve(1));
}
