import React from "react";
const P = Promise;
// Calling a global-object method can install a getter that later changes Promise.resolve.
globalThis.__defineGetter__("probe", function () {
  this.Promise.resolve = custom;
  return 1;
});
const observed = globalThis.probe;
const result = React.use(P.resolve(pending));
